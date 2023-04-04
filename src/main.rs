
mod util;
mod result;
mod ast;
mod scanner;
mod parser;

use result::Result;
use scanner::{Scanner, Token};

struct ResultIter<I> {
    iter: I,
}

impl <I: Iterator> Iterator for ResultIter<I> {
    type Item = Result<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(item) => Some(Ok(item)),
        }
    }
}

trait AsResult : Sized {
    fn as_result(self) -> ResultIter<Self>;
}

impl <I: Iterator> AsResult for I {
    fn as_result(self) -> ResultIter<Self> {
        ResultIter { iter: self }
    }
}

fn main() {
    let mut args = std::env::args();
    let path = args.nth(1).expect("could not find first argument");
    let text = std::fs::read_to_string(path).expect("could not read from file");
    let chars = text.chars().as_result();
    let mut scanner = Scanner::new(chars);
    let mut tokens = Vec::new();
    loop {
        let token = scanner.scan().unwrap();
        if let Token::EOF = token {
            break;
        }
        tokens.push(token);
    }
    eprintln!("{:#?}", tokens);
}
