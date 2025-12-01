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
impl AstNode {
    pub fn print(&self) {
        self.print_with_indent(0);
    }

    fn print_with_indent(&self, indent: usize) {
        let pad = "  ".repeat(indent);
        match self {
            AstNode::Program { args, vars, body, ret } => {
                println!("{pad}Program");
                println!("{pad}  Args: {:?}", args);
                println!("{pad}  Vars: {:?}", vars);
                println!("{pad}  Body:");
                for stmt in body {
                    stmt.print_with_indent(indent + 2);
                }
                println!("{pad}  Ret: {ret}");
            }

            AstNode::Assign { name, expr } => {
                println!("{pad}Assign {name}");
                expr.print_with_indent(indent + 1);
            }

            AstNode::If { cond, then_body, else_body } => {
                println!("{pad}If");
                println!("{pad}  Cond:");
                cond.print_with_indent(indent + 2);

                println!("{pad}  Then:");
                for stmt in then_body {
                    stmt.print_with_indent(indent + 2);
                }

                println!("{pad}  Else:");
                for stmt in else_body {
                    stmt.print_with_indent(indent + 2);
                }
            }

            AstNode::While { cond, body } => {
                println!("{pad}While");
                println!("{pad}  Cond:");
                cond.print_with_indent(indent + 2);
                println!("{pad}  Body:");
                for stmt in body {
                    stmt.print_with_indent(indent + 2);
                }
            }

            AstNode::BinaryOp { op, left, right } => {
                println!("{pad}BinaryOp {op}");
                left.print_with_indent(indent + 1);
                right.print_with_indent(indent + 1);
            }

            AstNode::BoolLiteral(val) => println!("{pad}BoolLiteral {val}"),
            AstNode::Identifier(name) => println!("{pad}Identifier {name}"),
            AstNode::Number(num) => println!("{pad}Number {num}"),
            AstNode::Empty => println!("{pad}Empty"),
            AstNode::Error => println!("{pad}Error"),
        }
    }
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
        let vars = self.parse_typedecl();
        let body = self.parse_stmts();
        let ret = self.parse_ret();
        if self.errors.is_empty() {
            return AstNode::Program { args, vars, body, ret };
        } else {
            return AstNode::Error;
        }
        AstNode::Program { args, vars, body, ret }
    }

    // ARGDECL -> args IDENTIFIER ARGDECLTAIL
    fn parse_argdecl(&mut self) -> Vec<String> {
        self.expect(&Token::Args, Some("Missing 'args' declaration")); 
        let mut args = Vec::new();
        self.parse_argdecltail(&mut args);
        args
    }

    // ARGDECLTAIL -> IDENTIFIER ( , IDENTIFIER )* ;
    fn parse_argdecltail(&mut self, args: &mut Vec<String>) {
        loop {
            match self.advance() {
                Some(Token::Identifier(name)) => {
                    args.push(name);
                    match self.peek() {
                        Some(Token::Comma) => { self.advance(); } // continue parsing more vars
                        Some(Token::Semicolon) => { self.advance(); return; }
                        _ => {
                            self.error("Expected ',' or ';' in argument declaration");
                            return;
                        }
                    }
                }
                Some(Token::Semicolon) => return,
                _ => {
                    self.error("Unexpected syntax in argument declaration");
                    return;
                }
            }
        }
    }

    // TYPEDECL -> type IDENTIFIER TYPEDECLTAIL
    fn parse_typedecl(&mut self) -> Vec<String> {
        self.expect(&Token::Int, Some("Missing keyword 'int' in variable declaration")); 
        let mut vars = Vec::new();
        self.parse_typedecltail(&mut vars);
        vars
    }

    // TYPEDECLTAIL -> IDENTIFIER ( , IDENTIFIER )* ;
    fn parse_typedecltail(&mut self, vars: &mut Vec<String>) {
        loop {
            match self.advance() {
                Some(Token::Identifier(name)) => {
                    vars.push(name);
                    match self.peek() {
                        Some(Token::Comma) => { self.advance(); } // more vars coming
                        Some(Token::Semicolon) => { self.advance(); return; }
                        _ => {
                            self.error("Expected ',' or ';' in variable declaration");
                            return;
                        }
                    }
                }
                Some(Token::Semicolon) => return,
                _ => {
                    self.error("Unexpected syntax in variable declaration");
                    return;
                }
            }
        }
    }

    // STMTS -> STMT STMTS | epsilon
    fn parse_stmts(&mut self) -> Vec<AstNode> {
        let mut stmts = Vec::new();

        loop {
            match self.peek() {
                // FIRST(STMT)
                Some(Token::Identifier(_))
                | Some(Token::If)
                | Some(Token::While) => {
                    let stmt = self.parse_stmt();
                    stmts.push(stmt);
                }

                // FOLLOW(STMTS)
                Some(Token::Return)
                | Some(Token::CClose)
                | Some(Token::EOF) => break,
                _ => {
                    break;
                }
            }
        }

        stmts
    }

    // STMT -> ASSIGN | IFTHENELSE | WHILE
    fn parse_stmt(&mut self) -> AstNode {
        match self.peek() {
            Some(Token::Identifier(_)) => self.parse_assign(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            other => {
                self.error(&format!("Unexpected token in statement: {:?}", other));
                AstNode::Error
            }
        }
    }


    // RET -> return IDENTIFER ;
    fn parse_ret(&mut self) -> String {
        self.expect(&Token::Return, Some("Missing 'return' statement"));
        // this could be EXPR instead of IDENTIFIER
        let ret_name = if let Some(Token::Identifier(name)) = self.advance() {
            name
        } else {
            self.error("Expected identifier after 'return'");
            "".to_string()
        };
        self.expect(&Token::Semicolon, None);
        ret_name
    }

    fn parse_assign(&mut self) -> AstNode {
        // stmt only calls this method if we know the token is IDENTIFIER
        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name
        } else {
            self.error("Expected identifier in assignment");
            return AstNode::Error;
        };
        self.expect(&Token::Assign, Some("Missing assignment operator"));
        let expr = self.parse_expr();
        self.expect(&Token::Semicolon, None);
        AstNode::Assign { name, expr: Box::new(expr) }
    }

    fn parse_if(&mut self) -> AstNode {
        self.expect(&Token::If, Some("Missing 'if' keyword (This should never be reached)"));
        let cond = self.parse_bool();
        self.expect(&Token::Then, Some("Missing 'then' keyword after if condition"));
        self.expect(&Token::COpen, Some("Missing '{' after then keyword"));
        let then_body = self.parse_stmts();
        self.expect(&Token::CClose, Some("Missing '}' after if body"));

        let else_body = if let Some(Token::Else) = self.peek() {
            self.advance();
            self.expect(&Token::COpen, Some("Missing '{' after else"));
            let body = self.parse_stmts();
            self.expect(&Token::CClose, Some("Missing '}' after else body"));
            body
        } else {
            Vec::new()
        };

        AstNode::If {
            cond: Box::new(cond),
            then_body,
            else_body,
        }
    }

    fn parse_while(&mut self) -> AstNode {
        self.expect(&Token::While, Some("Missing 'while' keyword (This should never be reached)"));
        let cond = self.parse_bool();
        self.expect(&Token::Then, Some("Missing 'then' keyword after while condition"));
        self.expect(&Token::COpen, Some("Missing '{' in while loop"));
        let body = self.parse_stmts();
        self.expect(&Token::CClose, Some("Missing '}' after while body"));
        AstNode::While { cond: Box::new(cond), body }
    }
    
    // BOOL -> true | false | EXPR BOOLDASH
    fn parse_bool(&mut self) -> AstNode {
        match self.peek() {
            Some(Token::True) => {
                self.advance();
                AstNode::BoolLiteral(true)
            }
            Some(Token::False) => {
                self.advance();
                AstNode::BoolLiteral(false)
            }
            _ => {
                let left = self.parse_expr();
                self.parse_booldash(left)
            }
        }
    }

    // BOOLDASH -> BOOLOP EXPR | epsilon
    fn parse_booldash(&mut self, left: AstNode) -> AstNode {
        match self.peek() {
            Some(Token::LE)
            | Some(Token::LThan)
            | Some(Token::GE)
            | Some(Token::GThan)
            | Some(Token::Equals) => {
                let op = match self.advance().unwrap() {
                    Token::LE => "<=".into(),
                    Token::LThan => "<".into(),
                    Token::GE => ">=".into(),
                    Token::GThan => ">".into(),
                    Token::Equals => "==".into(),
                    _ => unreachable!(),
                };

                let right = self.parse_expr();
                AstNode::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            _ => left, // ε case
        }
    }

    // EXPR -> TERM EXPRDASH
    fn parse_expr(&mut self) -> AstNode {
        let left = self.parse_term();
        self.parse_exprdash(left)
    }

    // EXPRDASH -> + TERM EXPRDASH | epsilon
    fn parse_exprdash(&mut self, mut left: AstNode) -> AstNode {
        loop {
            match self.peek() {
                Some(Token::Plus) | Some(Token::Minus) => {
                    let op = match self.advance().unwrap() {
                        Token::Plus => "+".into(),
                        Token::Minus => "-".into(),
                        _ => unreachable!(),
                    };
                    let right = self.parse_term();
                    left = AstNode::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        left
    }

    // TERM -> FACTOR TERMDASH
    fn parse_term(&mut self) -> AstNode {
        let left = self.parse_factor();
        self.parse_termdash(left)
    }

    // TERMDASH -> * FACTOR TERMDASH | epsilon
    fn parse_termdash(&mut self, mut left: AstNode) -> AstNode {
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    let op = match self.advance().unwrap() {
                        Token::Star => "*".into(),
                        _ => unreachable!(),
                    };
                    let right = self.parse_factor();
                    left = AstNode::BinaryOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        left
    }

    // FACTOR -> IDENTIFIER | NUMBER | (EXPR)
    fn parse_factor(&mut self) -> AstNode {
        match self.advance() {
            Some(Token::Identifier(name)) => AstNode::Identifier(name),
            Some(Token::Number(num)) => AstNode::Number(num),

            Some(Token::BOpen) => {
                let expr = self.parse_expr();
                if !self.expect(&Token::BClose, Some("Missing ')'")) {
                    self.error("Unclosed parenthesis in expression");
                }
                expr
            }

            Some(Token::True) => AstNode::BoolLiteral(true),
            Some(Token::False) => AstNode::BoolLiteral(false),

            Some(Token::Error(ch)) => {
                self.error(&format!("Invalid character '{}'", ch));
                AstNode::Error
            }

            Some(Token::EOF) | None => {
                self.error("Unexpected end of input");
                AstNode::Error
            }

            Some(t) => {
                self.error(&format!("Unexpected token in factor: {:?}", t));
                AstNode::Error
            }
        }
    }

}
