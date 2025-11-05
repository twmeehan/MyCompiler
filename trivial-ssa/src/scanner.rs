#[derive(Debug,Clone)]
pub enum Token {
    Identifier(String),
    Number(String),
    Plus,
    Star,
    Minus,
    Assign,
    LThan,
    GThan,
    LE,
    GE,
    Equals,
    BOpen,
    BClose,
    COpen,
    CClose,
    Comma,
    Semicolon,
    Args,
    Int,
    If,
    Then,
    Else,
    While,
    Return,
    True,
    False,
    Error(char),
    EOF,
}

pub struct Scanner {
    input: String,
    pos: usize,
}

impl Scanner {
    pub fn new(input: &str) -> Scanner {
        Scanner {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            while let Some(ch) = self.peek() {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            let token = match self.advance() {
                    Some('+') => Token::Plus,
		    Some('*') => Token::Star,
		    Some('-') => Token::Minus,
		    Some('(') => Token::BOpen,
		    Some(')') => Token::BClose,
		    Some('{') => Token::COpen,
		    Some('}') => Token::CClose,
		    Some(',') => Token::Comma,
		    Some(';') => Token::Semicolon,

		    Some('=') => {
			if self.peek() == Some('=') {
			    self.advance();
			    Token::Equals
			} else {
			    Token::Assign
			}
		    }
		    Some('<') => {
			if self.peek() == Some('=') {
			    self.advance();
			    Token::LE
			} else {
			    Token::LThan
			}
		    }
		    Some('>') => {
			if self.peek() == Some('=') {
			    self.advance();
			    Token::GE
			} else {
			    Token::GThan
			}
		    }//  if char is digit
                Some(ch) if ch.is_ascii_digit() => {
                    let mut num = ch.to_string();
                    while let Some(next) = self.peek() {
                        if next.is_ascii_digit() {
                            num.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    Token::Number(num)
                }
                // if char is ascii letters make an indentifier
                Some(ch) if ch.is_ascii_alphabetic() => {
                    let mut ident = ch.to_string();
                    while let Some(next) = self.peek() {
                        if next.is_ascii_alphabetic() {
                            ident.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
			let token = match ident.as_str() {
			    "args" => Token::Args,
			    "int" => Token::Int,
			    "if" => Token::If,
			    "then" => Token::Then,
			    "else" => Token::Else,
			    "while" => Token::While,
			    "return" => Token::Return,
			    "true" => Token::True,
			    "false" => Token::False,
			    _ => Token::Identifier(ident),
			};
			token
                }
                None => Token::EOF,
                Some(other) => {
                    println!("Warning: unexpected character '{}' at position {}", other, self.pos);
                    Token::Error(other)
                }
            };

            tokens.push(token.clone());
            if matches!(token, Token::EOF) {
                break;
            }
        }


        tokens
    }
}
