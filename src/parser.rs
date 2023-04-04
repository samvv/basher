
use crate::result::{Result, Error, Expected};
use crate::scanner::Token;
use crate::util::Stream;

pub struct Parser<T> {
    tokens: T,
}

enum Expr {
    Literal {
        text: String,
    },
    Command {
        parts: Vec<Expr>,
    },
}

enum Node {
    File {
        elements: Vec<Expr>,
    }
}

impl <T: Stream<Item = Result<Token>>> Parser<T> {

    pub fn parse_command(&mut self) -> Result<Expr> {
        let mut parts = Vec::new();
        loop {
            let t0 = self.tokens.get()?;
            match t0 {
                Token::EOF | Token::NewLine => break,
                Token::Ident(text) => parts.push(Expr::Literal { text }),
                Token::LParen => {
                    let inner = self.parse_command()?;
                    let t1 = self.tokens.get()?;
                    if !matches!(t1, Token::LParen) {
                        return Err(Error::UnexpectedToken { actual: t1, expected: Expected::RParen });
                    }
                },
                _ => return Err(Error::UnexpectedToken { actual: t0, expected: Expected::OneOf(vec![]) }),
            }
        }
        Ok(Expr::Command { parts })
    }

    pub fn parse_file(&mut self) -> Result<Node> {
        let mut commands = Vec::new();
        loop {
            commands.push(self.parse_command());
        }
    }

}
