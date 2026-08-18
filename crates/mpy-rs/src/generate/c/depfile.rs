use std::{ffi::OsString, path::PathBuf};

use anyhow::bail;

/// The single dependency rule emitted by Clang's `-MMD -MF` options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Depfile {
    pub targets: Vec<PathBuf>,
    pub dependencies: Vec<PathBuf>,
}

/// Parse a Clang Make-style dependency file.
///
/// Backslash escapes and Make's `$$` encoding are decoded, and escaped newline
/// continuations are treated as whitespace between paths.
pub fn parse(input: &[u8]) -> anyhow::Result<Depfile> {
    let mut parser = Parser { input, offset: 0 };
    let mut targets = Vec::new();

    loop {
        parser.skip_rule_whitespace();

        match parser.peek() {
            Some(b':') => {
                parser.offset += 1;
                break;
            }
            Some(b'\n' | b'\r' | b'#') | None => {
                bail!("invalid depfile: expected `:` at byte {}", parser.offset);
            }
            Some(_) => targets.push(parser.parse_path(true)?),
        }
    }

    if targets.is_empty() {
        bail!("invalid depfile: dependency rule has no target");
    }

    let mut dependencies = Vec::new();
    loop {
        parser.skip_rule_whitespace();

        match parser.peek() {
            Some(b'\n' | b'\r' | b'#') | None => break,
            Some(_) => dependencies.push(parser.parse_path(false)?),
        }
    }

    parser.skip_trailing_whitespace_and_comments();
    if parser.peek().is_some() {
        bail!(
            "invalid depfile: unexpected second dependency rule at byte {}",
            parser.offset
        );
    }

    Ok(Depfile {
        targets,
        dependencies,
    })
}

struct Parser<'a> {
    input: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.input.get(self.offset).copied()
    }

    fn skip_rule_whitespace(&mut self) {
        loop {
            match self.input.get(self.offset..) {
                Some([b' ' | b'\t', ..]) => self.offset += 1,
                Some([b'\\', b'\n', ..]) => self.offset += 2,
                Some([b'\\', b'\r', b'\n', ..]) => self.offset += 3,
                _ => break,
            }
        }
    }

    fn parse_path(&mut self, colon_terminates: bool) -> anyhow::Result<PathBuf> {
        let start = self.offset;
        let mut path = Vec::new();

        loop {
            match self.input.get(self.offset..) {
                Some([b'\\', b'\n', ..]) | Some([b'\\', b'\r', b'\n', ..]) => break,
                Some([b'\\', escaped, ..]) => {
                    path.push(*escaped);
                    self.offset += 2;
                }
                Some([b'\\']) => {
                    bail!("invalid depfile: dangling escape at byte {}", self.offset);
                }
                Some([b'$', b'$', ..]) => {
                    path.push(b'$');
                    self.offset += 2;
                }
                Some([byte, ..])
                    if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'#')
                        || (colon_terminates && *byte == b':') =>
                {
                    break;
                }
                Some([byte, ..]) => {
                    path.push(*byte);
                    self.offset += 1;
                }
                Some([]) | None => break,
            }
        }

        if path.is_empty() {
            bail!("invalid depfile: expected a path at byte {start}");
        }

        bytes_to_path(path)
    }

    fn skip_trailing_whitespace_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
                self.offset += 1;
            }

            if self.peek() != Some(b'#') {
                break;
            }

            while !matches!(self.peek(), Some(b'\n' | b'\r') | None) {
                self.offset += 1;
            }
        }
    }
}

#[cfg(unix)]
fn bytes_to_path(bytes: Vec<u8>) -> anyhow::Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;

    Ok(OsString::from_vec(bytes).into())
}

#[cfg(not(unix))]
fn bytes_to_path(bytes: Vec<u8>) -> anyhow::Result<PathBuf> {
    let path = String::from_utf8(bytes)
        .map_err(|error| anyhow::anyhow!("invalid UTF-8 in depfile path: {error}"))?;
    Ok(path.into())
}
