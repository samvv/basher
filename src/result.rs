
use crate::scanner::{Span, Token};

#[derive(Debug)]
pub enum Expected {
    OneOf(Vec<Expected>),
    RParen,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedToken {
        actual: Token,
        expected: Expected,
    },
    UnexpectedString {
        actual: String,
        span: Span,
    }
}

pub type Result<T> = std::result::Result<T, Error>;

