use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const CONFIG_ARRAY_NAME: &str = "mp_rs_config";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailParseError {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub context: String,
}

impl fmt::Display for TailParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {} (byte {}): {}",
            self.message, self.line, self.column, self.offset, self.context
        )
    }
}

impl Error for TailParseError {}

/// Parses the `mp_rs_config` array from preprocessed `tail.c` output.
pub fn parse_tail(input: &str) -> Result<BTreeMap<String, String>, TailParseError> {
    Parser::new(input).parse_array(CONFIG_ARRAY_NAME, true)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_array(
        mut self,
        array_name: &str,
        validate_config_names: bool,
    ) -> Result<BTreeMap<String, String>, TailParseError> {
        self.pos = self.find_identifier(array_name).ok_or_else(|| {
            self.error_at(self.input.len(), format!("`{array_name}` array not found"))
        })? + array_name.len();

        self.skip_trivia()?;
        self.expect_byte(b'[', &format!("expected `[` after `{array_name}`"))?;
        self.skip_trivia()?;
        self.expect_byte(b']', &format!("expected `]` in `{array_name}` declaration"))?;
        self.skip_trivia()?;
        self.expect_byte(
            b'=',
            &format!("expected `=` before `{array_name}` initializer"),
        )?;
        self.skip_trivia()?;
        self.expect_byte(
            b'{',
            &format!("expected `{{` to start `{array_name}` initializer"),
        )?;

        let mut entries = BTreeMap::new();
        loop {
            self.skip_trivia()?;
            if self.consume_byte(b'}') {
                break;
            }

            self.expect_byte(b'{', "expected `{` to start a config entry")?;
            self.skip_trivia()?;
            let name = self.parse_string_literal()?;
            if validate_config_names && !is_config_name(&name) {
                return Err(self.error(format!("invalid configuration name `{name}`")));
            }

            self.skip_trivia()?;
            self.expect_byte(b',', "expected `,` between config name and value")?;
            self.skip_trivia()?;
            let value = self.parse_string_literal()?;
            self.skip_trivia()?;
            self.expect_byte(b'}', "expected `}` after config entry")?;
            self.skip_trivia()?;
            self.expect_byte(b',', "expected `,` after config entry")?;

            if entries.insert(name.clone(), value).is_some() {
                return Err(self.error(format!("duplicate configuration entry `{name}`")));
            }
        }

        self.skip_trivia()?;
        self.expect_byte(
            b';',
            &format!("expected `;` after `{array_name}` initializer"),
        )?;
        Ok(entries)
    }

    fn parse_string_literal(&mut self) -> Result<String, TailParseError> {
        self.expect_byte(b'"', "expected a C string literal")?;
        let mut value = Vec::new();

        loop {
            let byte = self
                .next_byte()
                .ok_or_else(|| self.error("unterminated C string literal"))?;
            match byte {
                b'"' => {
                    return String::from_utf8(value)
                        .map_err(|_| self.error("C string literal is not valid UTF-8"));
                }
                b'\\' => self.parse_escape(&mut value)?,
                b'\n' | b'\r' => {
                    return Err(self.error("unescaped newline in C string literal"));
                }
                _ => value.push(byte),
            }
        }
    }

    fn parse_escape(&mut self, output: &mut Vec<u8>) -> Result<(), TailParseError> {
        let escaped = self
            .next_byte()
            .ok_or_else(|| self.error("unterminated escape sequence"))?;
        match escaped {
            b'\'' | b'"' | b'?' | b'\\' => output.push(escaped),
            b'a' => output.push(0x07),
            b'b' => output.push(0x08),
            b'f' => output.push(0x0c),
            b'n' => output.push(b'\n'),
            b'r' => output.push(b'\r'),
            b't' => output.push(b'\t'),
            b'v' => output.push(0x0b),
            b'\n' => {}
            b'\r' => {
                if self.peek_byte() == Some(b'\n') {
                    self.pos += 1;
                }
            }
            b'0'..=b'7' => {
                let mut number = u32::from(escaped - b'0');
                for _ in 0..2 {
                    let Some(next @ b'0'..=b'7') = self.peek_byte() else {
                        break;
                    };
                    self.pos += 1;
                    number = number * 8 + u32::from(next - b'0');
                }
                self.push_byte_escape(output, number)?;
            }
            b'x' => {
                let start = self.pos;
                let mut number = 0_u32;
                while let Some(digit) = self.peek_byte().and_then(hex_value) {
                    self.pos += 1;
                    number = number
                        .checked_mul(16)
                        .and_then(|value| value.checked_add(digit))
                        .ok_or_else(|| self.error("hexadecimal escape is too large"))?;
                }
                if self.pos == start {
                    return Err(self.error("hexadecimal escape has no digits"));
                }
                self.push_byte_escape(output, number)?;
            }
            b'u' => self.parse_unicode_escape(output, 4)?,
            b'U' => self.parse_unicode_escape(output, 8)?,
            _ => {
                return Err(self.error(format!(
                    "unsupported escape sequence `\\{}`",
                    escaped as char
                )));
            }
        }
        Ok(())
    }

    fn parse_unicode_escape(
        &mut self,
        output: &mut Vec<u8>,
        digits: usize,
    ) -> Result<(), TailParseError> {
        let mut number = 0_u32;
        for _ in 0..digits {
            let byte = self
                .next_byte()
                .ok_or_else(|| self.error("incomplete Unicode escape"))?;
            let digit = hex_value(byte)
                .ok_or_else(|| self.error("non-hexadecimal digit in Unicode escape"))?;
            number = number * 16 + digit;
        }

        let character = char::from_u32(number)
            .ok_or_else(|| self.error("invalid Unicode scalar value in escape"))?;
        let mut encoded = [0; 4];
        output.extend_from_slice(character.encode_utf8(&mut encoded).as_bytes());
        Ok(())
    }

    fn push_byte_escape(&self, output: &mut Vec<u8>, number: u32) -> Result<(), TailParseError> {
        let byte = u8::try_from(number)
            .map_err(|_| self.error("byte escape is outside the range 0..=255"))?;
        output.push(byte);
        Ok(())
    }

    fn skip_trivia(&mut self) -> Result<(), TailParseError> {
        loop {
            while self
                .peek_byte()
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                self.pos += 1;
            }

            if self.remaining().starts_with("//") {
                self.pos += 2;
                while let Some(byte) = self.next_byte() {
                    if byte == b'\n' {
                        break;
                    }
                }
            } else if self.remaining().starts_with("/*") {
                self.pos += 2;
                let length = self
                    .remaining()
                    .find("*/")
                    .ok_or_else(|| self.error("unterminated block comment"))?;
                self.pos += length + 2;
            } else {
                return Ok(());
            }
        }
    }

    fn find_identifier(&self, wanted: &str) -> Option<usize> {
        let bytes = self.input.as_bytes();
        let wanted_bytes = wanted.as_bytes();
        let mut start = 0;

        while start + wanted_bytes.len() <= bytes.len() {
            let relative = self.input[start..].find(wanted)?;
            let found = start + relative;
            let before = found.checked_sub(1).map(|index| bytes[index]);
            let after = bytes.get(found + wanted_bytes.len()).copied();
            if before.is_none_or(|byte| !is_identifier_byte(byte))
                && after.is_none_or(|byte| !is_identifier_byte(byte))
            {
                return Some(found);
            }
            start = found + wanted_bytes.len();
        }
        None
    }

    fn expect_byte(&mut self, expected: u8, message: &str) -> Result<(), TailParseError> {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.pos += 1;
        Some(byte)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn error(&self, message: impl Into<String>) -> TailParseError {
        self.error_at(self.pos, message)
    }

    fn error_at(&self, offset: usize, message: impl Into<String>) -> TailParseError {
        let prefix = &self.input[..offset];
        let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
        let column = prefix
            .rsplit_once('\n')
            .map_or(prefix.len() + 1, |(_, line)| line.len() + 1);
        let context_end = self.input[offset..]
            .char_indices()
            .nth(40)
            .map_or(self.input.len(), |(relative, _)| offset + relative);
        let context = self.input[offset..context_end].escape_debug().to_string();

        TailParseError {
            offset,
            line,
            column,
            message: message.into(),
            context: if context.is_empty() {
                "<end of input>".to_owned()
            } else {
                format!("near `{context}`")
            },
        }
    }
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_config_name(name: &str) -> bool {
    if name == "MP_INT_TYPE" {
        return true;
    }
    name.strip_prefix("MICROPY_")
        .or_else(|| name.strip_prefix("MP_INT_TYPE_"))
        .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(is_identifier_byte))
}

fn hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_tail;

    #[test]
    fn parses_preprocessed_tail() {
        let input = r#"
# 1 "tail.c"
struct mp_rs_config_entry { const char *name; const char *value; };
static const struct mp_rs_config_entry mp_rs_config[] = {
    { "MICROPY_EMPTY", "" },
    { "MICROPY_ENABLE_GC", "(1)" },
    { "MICROPY_HOOK", "do { foo(\"x\"); } while (0)" },
    { "MICROPY_VERSION_STRING", "\"1\" \".\" \"28\"" },
};
"#;

        let parsed = parse_tail(input).unwrap();
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed["MICROPY_EMPTY"], "");
        assert_eq!(parsed["MICROPY_ENABLE_GC"], "(1)");
        assert_eq!(parsed["MICROPY_HOOK"], "do { foo(\"x\"); } while (0)");
        assert_eq!(parsed["MICROPY_VERSION_STRING"], "\"1\" \".\" \"28\"");
    }

    #[test]
    fn accepts_comments_and_c_escapes() {
        let input = r#"
static int mp_rs_config_entry;
static const int mp_rs_config [ ] = {
    /* first */ { "MICROPY_TEXT", "line\\n\\x41\\101" }, // entry
};
"#;

        let parsed = parse_tail(input).unwrap();
        assert_eq!(parsed["MICROPY_TEXT"], "line\\n\\x41\\101");
    }

    #[test]
    fn rejects_duplicate_names() {
        let input = r#"
int mp_rs_config[] = {
    { "MICROPY_A", "0" },
    { "MICROPY_A", "1" },
};
"#;

        let error = parse_tail(input).unwrap_err();
        assert!(error.message.contains("duplicate"));
        assert!(error.line > 1);
    }

    #[test]
    fn reports_missing_array() {
        let error = parse_tail("int unrelated[] = {};").unwrap_err();
        assert!(error.message.contains("not found"));
        assert_eq!(error.offset, 21);
    }
}
