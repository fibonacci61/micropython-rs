use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalErrorKind {
    Unsupported,
    Syntax,
    Arithmetic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    pub kind: EvalErrorKind,
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}

impl Error for EvalError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluatedValue {
    Integer(i128),
    EmptyDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigValue {
    pub raw: String,
    pub evaluated: Result<EvaluatedValue, EvalError>,
}

impl ConfigValue {
    pub fn integer(&self) -> Option<i128> {
        match &self.evaluated {
            Ok(EvaluatedValue::Integer(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn is_empty_definition(&self) -> bool {
        self.evaluated == Ok(EvaluatedValue::EmptyDefinition)
    }
}

/// Evaluates every raw configuration value while retaining values that are not
/// integer constant expressions.
pub fn evaluate_config(raw_config: BTreeMap<String, String>) -> BTreeMap<String, ConfigValue> {
    raw_config
        .into_iter()
        .map(|(name, raw)| {
            let evaluated = if raw.is_empty() {
                Ok(EvaluatedValue::EmptyDefinition)
            } else {
                evaluate_integer_expression(&raw).map(EvaluatedValue::Integer)
            };
            (name, ConfigValue { raw, evaluated })
        })
        .collect()
}

/// Evaluates the integer subset of C preprocessor constant expressions.
///
/// Undefined `MICROPY_*` identifiers evaluate to zero, matching their behavior
/// in a preprocessor `#if`. Other unresolved identifiers are rejected so type,
/// pointer, and runtime-valued macros are not mistaken for integer cfg values.
pub fn evaluate_integer_expression(expression: &str) -> Result<i128, EvalError> {
    let mut parser = Parser::new(expression)?;
    let expression = parser.parse_conditional()?;
    if parser.current.kind != TokenKind::End {
        return Err(parser.error(
            EvalErrorKind::Syntax,
            "unexpected token after constant expression",
        ));
    }
    expression.evaluate()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr<'a> {
    Integer(i128),
    Identifier(&'a str, usize),
    Unary {
        operator: UnaryOperator,
        operand: Box<Expr<'a>>,
        offset: usize,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
        offset: usize,
    },
    Conditional {
        condition: Box<Expr<'a>>,
        if_true: Box<Expr<'a>>,
        if_false: Box<Expr<'a>>,
    },
}

impl Expr<'_> {
    fn evaluate(&self) -> Result<i128, EvalError> {
        match self {
            Self::Integer(value) => Ok(*value),
            Self::Identifier(name, _) if name.starts_with("MICROPY_") => Ok(0),
            Self::Identifier(name, offset) => Err(EvalError {
                kind: EvalErrorKind::Unsupported,
                offset: *offset,
                message: format!("unresolved non-configuration identifier `{name}`"),
            }),
            Self::Unary {
                operator,
                operand,
                offset,
            } => {
                let value = operand.evaluate()?;
                match operator {
                    UnaryOperator::Plus => Ok(value),
                    UnaryOperator::Minus => value
                        .checked_neg()
                        .ok_or_else(|| arithmetic_error(*offset, "integer overflow in unary `-`")),
                    UnaryOperator::LogicalNot => Ok(i128::from(value == 0)),
                    UnaryOperator::BitwiseNot => Ok(!value),
                }
            }
            Self::Binary {
                operator,
                left,
                right,
                offset,
            } => evaluate_binary(*operator, left, right, *offset),
            Self::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                if condition.evaluate()? != 0 {
                    if_true.evaluate()
                } else {
                    if_false.evaluate()
                }
            }
        }
    }
}

fn evaluate_binary(
    operator: BinaryOperator,
    left: &Expr<'_>,
    right: &Expr<'_>,
    offset: usize,
) -> Result<i128, EvalError> {
    match operator {
        BinaryOperator::LogicalAnd => {
            let left = left.evaluate()?;
            Ok(i128::from(left != 0 && right.evaluate()? != 0))
        }
        BinaryOperator::LogicalOr => {
            let left = left.evaluate()?;
            Ok(i128::from(left != 0 || right.evaluate()? != 0))
        }
        _ => {
            let left = left.evaluate()?;
            let right = right.evaluate()?;
            match operator {
                BinaryOperator::Multiply => left
                    .checked_mul(right)
                    .ok_or_else(|| arithmetic_error(offset, "integer overflow in `*`")),
                BinaryOperator::Divide if right == 0 => {
                    Err(arithmetic_error(offset, "division by zero"))
                }
                BinaryOperator::Divide => left
                    .checked_div(right)
                    .ok_or_else(|| arithmetic_error(offset, "integer overflow in `/`")),
                BinaryOperator::Remainder if right == 0 => {
                    Err(arithmetic_error(offset, "remainder by zero"))
                }
                BinaryOperator::Remainder => left
                    .checked_rem(right)
                    .ok_or_else(|| arithmetic_error(offset, "integer overflow in `%`")),
                BinaryOperator::Add => left
                    .checked_add(right)
                    .ok_or_else(|| arithmetic_error(offset, "integer overflow in `+`")),
                BinaryOperator::Subtract => left
                    .checked_sub(right)
                    .ok_or_else(|| arithmetic_error(offset, "integer overflow in `-`")),
                BinaryOperator::ShiftLeft => checked_shift(right)
                    .and_then(|shift| left.checked_shl(shift))
                    .ok_or_else(|| arithmetic_error(offset, "invalid or overflowing left shift")),
                BinaryOperator::ShiftRight => checked_shift(right)
                    .and_then(|shift| left.checked_shr(shift))
                    .ok_or_else(|| arithmetic_error(offset, "invalid right shift")),
                BinaryOperator::Less => Ok(i128::from(left < right)),
                BinaryOperator::LessEqual => Ok(i128::from(left <= right)),
                BinaryOperator::Greater => Ok(i128::from(left > right)),
                BinaryOperator::GreaterEqual => Ok(i128::from(left >= right)),
                BinaryOperator::Equal => Ok(i128::from(left == right)),
                BinaryOperator::NotEqual => Ok(i128::from(left != right)),
                BinaryOperator::BitwiseAnd => Ok(left & right),
                BinaryOperator::BitwiseXor => Ok(left ^ right),
                BinaryOperator::BitwiseOr => Ok(left | right),
                BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => unreachable!(),
            }
        }
    }
}

fn checked_shift(value: i128) -> Option<u32> {
    let shift = u32::try_from(value).ok()?;
    (shift < i128::BITS).then_some(shift)
}

fn arithmetic_error(offset: usize, message: &str) -> EvalError {
    EvalError {
        kind: EvalErrorKind::Arithmetic,
        offset,
        message: message.to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnaryOperator {
    Plus,
    Minus,
    LogicalNot,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryOperator {
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
}

impl BinaryOperator {
    fn precedence(self) -> u8 {
        match self {
            Self::LogicalOr => 1,
            Self::LogicalAnd => 2,
            Self::BitwiseOr => 3,
            Self::BitwiseXor => 4,
            Self::BitwiseAnd => 5,
            Self::Equal | Self::NotEqual => 6,
            Self::Less | Self::LessEqual | Self::Greater | Self::GreaterEqual => 7,
            Self::ShiftLeft | Self::ShiftRight => 8,
            Self::Add | Self::Subtract => 9,
            Self::Multiply | Self::Divide | Self::Remainder => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Token<'a> {
    kind: TokenKind<'a>,
    offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind<'a> {
    Integer(i128),
    Identifier(&'a str),
    LeftParen,
    RightParen,
    Question,
    Colon,
    Plus,
    Minus,
    Bang,
    Tilde,
    Star,
    Slash,
    Percent,
    ShiftLeft,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    Ampersand,
    Caret,
    Pipe,
    LogicalAnd,
    LogicalOr,
    End,
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token<'a>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self, EvalError> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self { lexer, current })
    }

    fn parse_conditional(&mut self) -> Result<Expr<'a>, EvalError> {
        let condition = self.parse_binary(1)?;
        if self.current.kind != TokenKind::Question {
            return Ok(condition);
        }

        self.advance()?;
        let if_true = self.parse_conditional()?;
        self.expect(TokenKind::Colon, "expected `:` in conditional expression")?;
        let if_false = self.parse_conditional()?;
        Ok(Expr::Conditional {
            condition: Box::new(condition),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        })
    }

    fn parse_binary(&mut self, minimum_precedence: u8) -> Result<Expr<'a>, EvalError> {
        let mut left = self.parse_unary()?;
        loop {
            let Some(operator) = binary_operator(self.current.kind) else {
                break;
            };
            let precedence = operator.precedence();
            if precedence < minimum_precedence {
                break;
            }

            let offset = self.current.offset;
            self.advance()?;
            let right = self.parse_binary(precedence + 1)?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
                offset,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr<'a>, EvalError> {
        let operator = match self.current.kind {
            TokenKind::Plus => Some(UnaryOperator::Plus),
            TokenKind::Minus => Some(UnaryOperator::Minus),
            TokenKind::Bang => Some(UnaryOperator::LogicalNot),
            TokenKind::Tilde => Some(UnaryOperator::BitwiseNot),
            _ => None,
        };
        if let Some(operator) = operator {
            let offset = self.current.offset;
            self.advance()?;
            return Ok(Expr::Unary {
                operator,
                operand: Box::new(self.parse_unary()?),
                offset,
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr<'a>, EvalError> {
        let token = self.current;
        match token.kind {
            TokenKind::Integer(value) => {
                self.advance()?;
                Ok(Expr::Integer(value))
            }
            TokenKind::Identifier(name) => {
                self.advance()?;
                if self.current.kind == TokenKind::LeftParen {
                    return Err(EvalError {
                        kind: EvalErrorKind::Unsupported,
                        offset: token.offset,
                        message: format!(
                            "function-like expression `{name}(...)` is not an integer cfg expression"
                        ),
                    });
                }
                Ok(Expr::Identifier(name, token.offset))
            }
            TokenKind::LeftParen => {
                self.advance()?;
                let expression = self.parse_conditional()?;
                self.expect(TokenKind::RightParen, "expected `)`")?;
                Ok(expression)
            }
            TokenKind::End => Err(self.error(EvalErrorKind::Syntax, "expected an expression")),
            _ => Err(self.error(
                EvalErrorKind::Unsupported,
                "unsupported token in integer constant expression",
            )),
        }
    }

    fn expect(&mut self, expected: TokenKind<'a>, message: &str) -> Result<(), EvalError> {
        if self.current.kind == expected {
            self.advance()
        } else {
            Err(self.error(EvalErrorKind::Syntax, message))
        }
    }

    fn advance(&mut self) -> Result<(), EvalError> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    fn error(&self, kind: EvalErrorKind, message: &str) -> EvalError {
        EvalError {
            kind,
            offset: self.current.offset,
            message: message.to_owned(),
        }
    }
}

fn binary_operator(token: TokenKind<'_>) -> Option<BinaryOperator> {
    Some(match token {
        TokenKind::Star => BinaryOperator::Multiply,
        TokenKind::Slash => BinaryOperator::Divide,
        TokenKind::Percent => BinaryOperator::Remainder,
        TokenKind::Plus => BinaryOperator::Add,
        TokenKind::Minus => BinaryOperator::Subtract,
        TokenKind::ShiftLeft => BinaryOperator::ShiftLeft,
        TokenKind::ShiftRight => BinaryOperator::ShiftRight,
        TokenKind::Less => BinaryOperator::Less,
        TokenKind::LessEqual => BinaryOperator::LessEqual,
        TokenKind::Greater => BinaryOperator::Greater,
        TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
        TokenKind::EqualEqual => BinaryOperator::Equal,
        TokenKind::NotEqual => BinaryOperator::NotEqual,
        TokenKind::Ampersand => BinaryOperator::BitwiseAnd,
        TokenKind::Caret => BinaryOperator::BitwiseXor,
        TokenKind::Pipe => BinaryOperator::BitwiseOr,
        TokenKind::LogicalAnd => BinaryOperator::LogicalAnd,
        TokenKind::LogicalOr => BinaryOperator::LogicalOr,
        _ => return None,
    })
}

struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn next_token(&mut self) -> Result<Token<'a>, EvalError> {
        self.skip_whitespace();
        let offset = self.position;
        let Some(byte) = self.peek() else {
            return Ok(Token {
                kind: TokenKind::End,
                offset,
            });
        };

        if byte.is_ascii_digit() {
            return self.integer_token();
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            self.position += 1;
            while self
                .peek()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                self.position += 1;
            }
            return Ok(Token {
                kind: TokenKind::Identifier(&self.input[offset..self.position]),
                offset,
            });
        }

        for (operator, kind) in [
            ("<<", TokenKind::ShiftLeft),
            (">>", TokenKind::ShiftRight),
            ("<=", TokenKind::LessEqual),
            (">=", TokenKind::GreaterEqual),
            ("==", TokenKind::EqualEqual),
            ("!=", TokenKind::NotEqual),
            ("&&", TokenKind::LogicalAnd),
            ("||", TokenKind::LogicalOr),
        ] {
            if self.remaining().starts_with(operator) {
                self.position += operator.len();
                return Ok(Token { kind, offset });
            }
        }

        self.position += 1;
        let kind = match byte {
            b'(' => TokenKind::LeftParen,
            b')' => TokenKind::RightParen,
            b'?' => TokenKind::Question,
            b':' => TokenKind::Colon,
            b'+' => TokenKind::Plus,
            b'-' => TokenKind::Minus,
            b'!' => TokenKind::Bang,
            b'~' => TokenKind::Tilde,
            b'*' => TokenKind::Star,
            b'/' => TokenKind::Slash,
            b'%' => TokenKind::Percent,
            b'<' => TokenKind::Less,
            b'>' => TokenKind::Greater,
            b'&' => TokenKind::Ampersand,
            b'^' => TokenKind::Caret,
            b'|' => TokenKind::Pipe,
            b'"' | b'\'' => {
                return Err(EvalError {
                    kind: EvalErrorKind::Unsupported,
                    offset,
                    message: "string and character literals are not integer cfg expressions"
                        .to_owned(),
                });
            }
            _ => {
                return Err(EvalError {
                    kind: EvalErrorKind::Syntax,
                    offset,
                    message: format!("unexpected character `{}`", byte as char),
                });
            }
        };
        Ok(Token { kind, offset })
    }

    fn integer_token(&mut self) -> Result<Token<'a>, EvalError> {
        let offset = self.position;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.position += 1;
        }
        let literal = &self.input[offset..self.position];
        let value = parse_integer_literal(literal).map_err(|message| EvalError {
            kind: EvalErrorKind::Syntax,
            offset,
            message,
        })?;
        Ok(Token {
            kind: TokenKind::Integer(value),
            offset,
        })
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.position).copied()
    }

    fn remaining(&self) -> &str {
        &self.input[self.position..]
    }
}

fn parse_integer_literal(literal: &str) -> Result<i128, String> {
    let (base, prefix_length) = if literal.starts_with("0x") || literal.starts_with("0X") {
        (16, 2)
    } else if literal.starts_with("0b") || literal.starts_with("0B") {
        (2, 2)
    } else if literal.starts_with('0') && literal.len() > 1 {
        (8, 0)
    } else {
        (10, 0)
    };

    let body = &literal[prefix_length..];
    let digit_count = body
        .bytes()
        .take_while(|byte| digit_value(*byte).is_some_and(|digit| digit < base))
        .count();
    if digit_count == 0 {
        return Err(format!("integer literal `{literal}` has no digits"));
    }

    let digits = &body[..digit_count];
    let suffix = &body[digit_count..];
    if !valid_integer_suffix(suffix) {
        return Err(format!("invalid integer suffix in `{literal}`"));
    }

    i128::from_str_radix(digits, base)
        .map_err(|_| format!("integer literal `{literal}` is outside the supported range"))
}

fn digit_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

fn valid_integer_suffix(suffix: &str) -> bool {
    matches!(
        suffix.to_ascii_lowercase().as_str(),
        "" | "u" | "l" | "ul" | "lu" | "ll" | "ull" | "llu"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn eval(expression: &str) -> i128 {
        evaluate_integer_expression(expression).unwrap()
    }

    #[test]
    fn evaluates_precedence_and_integer_literals() {
        assert_eq!(eval("1 + 2 * 3"), 7);
        assert_eq!(eval("(1 + 2) * 3"), 9);
        assert_eq!(eval("0xffU + 010 + 0b10"), 265);
        assert_eq!(eval("1 << 16 | 26 << 8 | 0"), 72_192);
    }

    #[test]
    fn evaluates_comparison_boolean_and_bitwise_operators() {
        assert_eq!(eval("30 >= 10 && 2 != 3"), 1);
        assert_eq!(eval("0 || 0 || 4"), 1);
        assert_eq!(eval("!0 + !!7"), 2);
        assert_eq!(eval("(~0 & 15) ^ 3"), 12);
    }

    #[test]
    fn evaluates_conditional_and_short_circuit_expressions() {
        assert_eq!(eval("0 ? 1 / 0 : 42"), 42);
        assert_eq!(eval("1 ? 7 : 1 / 0"), 7);
        assert_eq!(eval("0 && (1 / 0)"), 0);
        assert_eq!(eval("1 || (1 / 0)"), 1);
    }

    #[test]
    fn treats_undefined_micropython_identifiers_as_zero() {
        assert_eq!(eval("MICROPY_EMIT_NATIVE_DEBUG || 1"), 1);
        assert_eq!(eval("MICROPY_PY_SSL_ECDSA_SIGN_ALT"), 0);
    }

    #[test]
    fn handles_empty_macro_definitions_explicitly() {
        let config = evaluate_config(BTreeMap::from([
            ("MICROPY_HOOK".to_owned(), String::new()),
            ("MICROPY_VALUE".to_owned(), "(1)".to_owned()),
        ]));

        assert_eq!(
            config["MICROPY_HOOK"].evaluated,
            Ok(EvaluatedValue::EmptyDefinition)
        );
        assert!(config["MICROPY_HOOK"].is_empty_definition());
        assert_eq!(config["MICROPY_VALUE"].integer(), Some(1));
    }

    #[test]
    fn preserves_non_integer_values_as_errors() {
        for expression in [
            "",
            "size_t",
            "&mp_plat_print",
            "sizeof(mp_uint_t)",
            "\"MicroPython\"",
            "MP_SMALL_INT_POSITIVE_MASK + 1",
        ] {
            let error = evaluate_integer_expression(expression).unwrap_err();
            assert!(matches!(
                error.kind,
                EvalErrorKind::Unsupported | EvalErrorKind::Syntax
            ));
        }
    }

    #[test]
    fn reports_arithmetic_errors() {
        let error = evaluate_integer_expression("1 / 0").unwrap_err();
        assert_eq!(error.kind, EvalErrorKind::Arithmetic);
        assert_eq!(error.offset, 2);
    }
}
