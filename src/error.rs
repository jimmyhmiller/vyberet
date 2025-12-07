//! Parse error types and error handling

use crate::tokenizer::{Token, TokenType};
use thiserror::Error;

/// Parse errors
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Expected {expected:?} but found {found:?} at {location}")]
    Expected {
        expected: TokenType,
        found: TokenType,
        location: String,
    },

    #[error("Unexpected token {token:?} at {location}")]
    UnexpectedToken { token: TokenType, location: String },

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Invalid {what} at {location}: {message}")]
    Invalid {
        what: String,
        location: String,
        message: String,
    },

    #[error("Parse error at {location}: {message}")]
    General { location: String, message: String },

    #[error("Operators of different kinds cannot be mixed at the same level: `{op_a}` at {loc_a} cannot be mixed with `{op_b}` at {loc_b}. Use parentheses to group the operations.")]
    MixedBinops {
        op_a: String,
        loc_a: String,
        op_b: String,
        loc_b: String,
    },
}

impl ParseError {
    pub fn expected(expected: TokenType, found_token: Token) -> Self {
        ParseError::Expected {
            expected,
            found: found_token.token_type.clone(),
            location: format_location(&found_token),
        }
    }

    pub fn unexpected(token: Token) -> Self {
        ParseError::UnexpectedToken {
            token: token.token_type.clone(),
            location: format_location(&token),
        }
    }

    pub fn invalid(what: &str, token: &Token, message: &str) -> Self {
        ParseError::Invalid {
            what: what.to_string(),
            location: format_location(token),
            message: message.to_string(),
        }
    }

    pub fn general(token: &Token, message: &str) -> Self {
        ParseError::General {
            location: format_location(token),
            message: message.to_string(),
        }
    }

    pub fn mixed_binops(op_a: &str, loc_a: &Token, op_b: &str, loc_b: &Token) -> Self {
        ParseError::MixedBinops {
            op_a: op_a.to_string(),
            loc_a: format_location(loc_a),
            op_b: op_b.to_string(),
            loc_b: format_location(loc_b),
        }
    }
}

fn format_location(token: &Token) -> String {
    format!(
        "{}:{}:{}",
        token.location.start_line, token.location.start_column, token.location.start_char
    )
}

pub type ParseResult<T> = Result<T, ParseError>;
