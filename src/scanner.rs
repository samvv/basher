
use std::collections::VecDeque;

use super::result::{Result, Error};

const EOF: char = '\u{FFFF}';

fn is_whitespace(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\t' | ' ')
}

pub struct Scanner<C> {
    chars: C,
    char_buffer: VecDeque<char>,
    offset: usize,
    start_offset: usize,
    mode: ScanMode,
}

pub type Span = std::ops::Range<usize>;
  
#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Ident(String), 
    Text(String),
    Dollar,
    DollarDollar,
    DollarIdent(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Integer(i64),
    IfKeyword,
    ElifKeyword,
    ElseKeyword,
    FiKeyword,
    ForKeyword,
    WhileKeyword,
    UntilKeyword,
    CaseKeyword,
    EsacKeyword,
    FunctionKeyword,
    DoKeyword,
    DoneKeyword,
    Semi,
    SemiSemi,
    EOF,
    NewLine,
}

enum ScanMode {
    Text,
    Expr,
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || matches!(ch, '_')
}

fn is_ident_part(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_')
}

impl <C: Iterator<Item = Result<char>>> Scanner<C> {

    pub fn new(chars: C) -> Self {
        Self {
            chars,
            char_buffer: VecDeque::new(),
            offset: 0,
            start_offset: 0,
            mode: ScanMode::Text,
        }
    }

    fn read_char(&mut self) -> Result<char> {
        match self.chars.next() {
            None => Ok(EOF),
            Some(result) => result,
        }
    }

    fn peek_char(&mut self, count: usize) -> Result<char> {
        let n = self.char_buffer.len();
        let k = if count > n { count-n } else { 0 };
        for _ in 0..k {
            let ch = self.read_char()?;
            self.char_buffer.push_back(ch);
        }
        Ok(*self.char_buffer.iter().nth(count-1).unwrap())
    }

    fn skip_char(&mut self) -> Result<()> {
        match self.char_buffer.pop_front() {
            None => { self.chars.next(); Ok(()) },
            Some(_) => Ok(()),
        }
    }

    fn skip_chars(&mut self, count: usize) -> Result<()> {
        if count > 0 {
            let n = std::cmp::min(self.char_buffer.len(), count);
            for _ in 0..n {
                self.char_buffer.pop_front();
            }
            for _ in 0..count-n {
                if self.chars.next().is_none() {
                    break;
                }
            }
        }
        Ok(())
    }

    fn get_char(&mut self) -> Result<char> {
        match self.char_buffer.pop_front() {
            None => self.read_char(),
            Some(ch) => Ok(ch),
        }
    }

    fn take_while(&mut self, pred: fn (char) -> bool, out: &mut String) -> Result<()> {
        loop {
            let c0 = self.peek_char(1)?;
            if !pred(c0) {
                break;
            }
            self.get_char()?;
            out.push(c0);
        }
        Ok(())
    }

    pub fn set_mode(&mut self, mode: ScanMode) {
        self.mode = mode;
    }

    pub fn span(&self) -> Span {
        self.start_offset..self.offset
    }

    fn scan_ident(&mut self) -> Result<String> {
        let mut text = String::new();
        text.push(self.get_char()?);
        self.take_while(is_ident_part, &mut text)?;
        Ok(text)
    }

    pub fn scan(&mut self) -> Result<Token> {
        let mut buffer = String::new();
        macro_rules! token { 
            ($expr:expr) => {
                {
                    if !buffer.is_empty() {
                        return Ok(Token::Text(std::mem::take(&mut buffer)));
                    }
                    return Ok($expr);
                }
            };
        }
        loop {
            let c0 = self.peek_char(1)?;
            match self.mode {
                ScanMode::Text => match c0 {
                    ';' => {
                        let c1 = self.peek_char(2)?;
                        token!(
                            match c1 {
                                ';' => { self.skip_chars(2)?; Token::SemiSemi },
                                _ => { self.skip_char()?; Token::Semi },
                            }
                        );
                    },
                    '(' => token!({ self.skip_char()?; Token::LParen }),
                    ')' => token!({ self.skip_char()?; Token::RParen }),
                    '[' => token!({ self.skip_char()?; Token::LBracket }),
                    ']' => token!({ self.skip_char()?; Token::RBracket }),
                    '$' => {
                        let c1 = self.peek_char(2)?;
                        token!({
                            self.skip_chars(1)?;
                            match c1 {
                                '$' => { self.skip_char()?; Token::DollarDollar },
                                ch if is_ident_start(ch) => Token::DollarIdent(self.scan_ident()?),
                                _ => Token::Dollar
                            }
                        });
                    }
                    EOF => {
                        token!(Token::EOF);
                    },
                    _ => {
                        self.skip_chars(1)?;
                        buffer.push(c0);
                    }
                }
                ScanMode::Expr => match c0 {
                    EOF => return Ok(Token::EOF),
                    ch if ch.is_numeric() => {
                        let mut value: i64 = c0.to_digit(10).unwrap() as i64;
                        self.skip_char()?;
                        loop {
                            let c1 = self.peek_char(1)?;
                            if !c1.is_numeric() {
                                break;
                            }
                            value = value * 10 + (c1.to_digit(10).unwrap() as i64);
                            self.skip_char()?;
                        }
                        return Ok(Token::Integer(value))
                    },
                    ch if is_ident_start(ch) => {
                        let mut text = String::new();
                        text.push(c0);
                        self.skip_char()?;
                        self.take_while(is_ident_part, &mut text)?;
                        match text.as_str() {
                            "if" => return Ok(Token::IfKeyword),
                            "fi" => return Ok(Token::FiKeyword),
                            "for" => return Ok(Token::ForKeyword),
                            "while" => return Ok(Token::WhileKeyword),
                            "do" => return Ok(Token::DoKeyword),
                            "done" => return Ok(Token::DoneKeyword),
                            "case" => return Ok(Token::CaseKeyword),
                            "esac" => return Ok(Token::EsacKeyword),
                            _ => return Ok(Token::Ident(text)),
                        }
                    },
                    _ => return Err(Error::UnexpectedString { actual: c0.to_string(), span: self.start_offset..self.offset }),
                }
            }
        }
    }

}

#[cfg(test)]
mod test {

    use super::{Token, Scanner};

    #[test]
    fn test_scan_text() {
        let mut scanner = Scanner::new("foo bar baz".chars().map(|ch| Ok(ch)));
        let t0 = scanner.scan().unwrap();
        assert_eq!(t0, Token::Text("foo bar baz".to_string()));
    }

    #[test]
    fn test_scan_dollar() {
        let mut scanner = Scanner::new("$foo baz $bar".chars().map(|ch| Ok(ch)));
        let t1 = scanner.scan().unwrap();
        assert_eq!(t1, Token::DollarIdent("foo".to_string()));
        let t2 = scanner.scan().unwrap();
        assert_eq!(t2, Token::Text(" baz ".to_string()));
        let t3 = scanner.scan().unwrap();
        assert_eq!(t3, Token::DollarIdent("bar".to_string()));
    }

}
