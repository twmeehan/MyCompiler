use crate::scanner::Token;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum AstNode {
    Program {
        args: Vec<String>,
        vars: Vec<String>,
        body: Vec<AstNode>,
        ret: String,
    },
    Assign { name: String, expr: Box<AstNode> },
    If {
        cond: Box<AstNode>,
        then_body: Vec<AstNode>,
        else_body: Vec<AstNode>,
    },
    While {
        cond: Box<AstNode>,
        body: Vec<AstNode>,
    },
    BinaryOp {
        op: String,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    BoolLiteral(bool),
    Identifier(String),
    Number(String),
    Empty,
    Error,
}

pub struct Parser {
    tokens: VecDeque<Token>,
    pub errors: Vec<String>,
    pub ast: Option<AstNode>,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        let mut parser = Parser {
            tokens: VecDeque::from(tokens.to_vec()),
            errors: Vec::new(),
            ast: None,
        };

        let result = parser.parse_program();
        if parser.errors.is_empty() {
            parser.ast = Some(result);
        }
        parser
    }

    fn peek(&self) -> Option<&Token> {self.tokens.front()}
    fn advance(&mut self) -> Option<Token> {self.tokens.pop_front()}
    fn error(&mut self, msg: &str) { self.errors.push(msg.to_string());}
    fn expect(&mut self, expected: &Token, msg: Option<&str>) -> bool {
        if let Some(token) = self.peek() {
            if token == expected {
                self.advance();
                return true;
            }
        }
        if let Some(m) = msg {
            self.error(m);
        } else {
            self.error(&format!("Expected {:?} but found {:?}", expected, self.peek()));
        }
        false
    }

    // PROG -> ARGDECL TYPEDECL STMTS RET
    fn parse_program(&mut self) -> AstNode {
        let args = self.parse_argdecl();
        let vars = vec![];//self.parse_typedecl();
        let body = vec![];//self.parse_stmts();
        let ret = self.parse_ret();
        if self.errors.is_empty() {
            AstNode::Program { args, vars, body, ret }
        } else {
            AstNode::Error
        }
        AstNode::Program { args, vars, body, ret }
    }

    // ARGDECL -> args IDENTIFIER ARGDECLTAIL
    fn parse_argdecl(&mut self) -> Vec<String> {
        self.expect(&Token::Args, "Missing 'args' declaration"); 
        let mut args = Vec::new();
        self.parse_argdecltail(&mut args);
        args
    }

    // ARGDECLTAIL -> ; | IDENTIFIER ARGDECLTAIL
    fn parse_argdecltail(&mut self, args: &mut Vec<String>) {
        match self.advance() {
            Some(Token::Identifier(name)) => {
                args.push(name);
                self.parse_argdecltail(args);
            }
            Some(Token::Semicolon) => {
                return;
            }
            _ => {
                self.error("Unexpected syntax in argument declaration");
                return;
            }
        }
    }
    
    // RET -> return IDENTIFER ;
    fn parse_ret(&mut self) -> String {
        self.expect(&Token::Return, "Missing 'return' statement");
        // this could be EXPR instead of IDENTIFIER
        let ret_name = if let Some(Token::Identifier(name)) = self.advance() {
            name
        } else {
            self.error("Expected identifier after 'return'");
            "".to_string()
        };
        self.expect(&Token::Semicolon);
        ret_name
    }


    // fn parse_expr(&mut self) -> AstNode {
    //     let mut left = self.parse_term();

    //     loop {
    //         match self.peek() {
    //             Some(Token::Plus) => {
    //                 self.advance();
    //                 let right = self.parse_term();
    //                 left = AstNode::BinaryOp {
    //                     op: "+".into(),
    //                     left: Box::new(left),
    //                     right: Box::new(right),
    //                 };
    //             }
    //             Some(Token::Minus) => {
    //                 self.advance();
    //                 let right = self.parse_term();
    //                 left = AstNode::BinaryOp {
    //                     op: "-".into(),
    //                     left: Box::new(left),
    //                     right: Box::new(right),
    //                 };
    //             }
    //             _ => break,
    //         }
    //     }

    //     left
    // }

    // fn parse_term(&mut self) -> AstNode {
    //     let mut left = self.parse_factor();

    //     loop {
    //         match self.peek() {
    //             Some(Token::Star) => {
    //                 self.advance();
    //                 let right = self.parse_factor();
    //                 left = AstNode::BinaryOp {
    //                     op: "*".into(),
    //                     left: Box::new(left),
    //                     right: Box::new(right),
    //                 };
    //             }
    //             _ => break,
    //         }
    //     }

    //     left
    // }

    // fn parse_factor(&mut self) -> AstNode {
    //     match self.advance() {
    //         Some(Token::Identifier(id)) => AstNode::Identifier(id),
    //         Some(Token::Number(n)) => AstNode::Number(n),

    //         Some(Token::BOpen) => {
    //             let expr = self.parse_expr();
    //             match self.advance() {
    //                 Some(Token::BClose) => expr,
    //                 other => {
    //                     self.error(format!("Expected ')', found {:?}", other));
    //                     AstNode::Error
    //                 }
    //             }
    //         }

    //         Some(Token::Error(ch)) => {
    //             self.error(format!("Invalid character '{}'", ch));
    //             AstNode::Error
    //         }

    //         Some(Token::EOF) | None => {
    //             self.error("Unexpected end of input");
    //             AstNode::Error
    //         }

    //         Some(t) => {
    //             self.error(format!("Unexpected token: {:?}", t));
    //             AstNode::Error
    //         }
    //     }
    // }
}
