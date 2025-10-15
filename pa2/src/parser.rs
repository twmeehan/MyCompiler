use crate::scanner::Token;
use std::collections::VecDeque;

pub struct ParseTree {
    pub label: String,
    pub children: Vec<ParseTree>,
}

pub struct ParseError {
    pub message: String,
}

pub fn report_error(errors: &mut Vec<ParseError>, msg: &str) {
    eprintln!("Parse error: {}", msg);
    errors.push(ParseError {
        message: msg.to_string(),
    });
}

impl ParseTree {
    pub fn new(label: &str) -> ParseTree {
        ParseTree {
            label: label.to_string(),
            children: Vec::new(),
        }
    }

    pub fn print(&self) {
        // Breadth-first traversal
        let mut queue: Vec<&ParseTree> = vec![self];
        while !queue.is_empty() {
            let mut next: Vec<&ParseTree> = Vec::new();
            for node in &queue {
                print!("{} ", node.label);
                for child in &node.children {
                    next.push(child);
                }
            }
            println!();
            queue = next;
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Number(String),
    Identifier(String),
    BinaryOp {
        op: String,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    Empty,
    Error,
}

impl AstNode {
    // Breadth-first traversal like parse tree
    pub fn print(&self) {
        use std::collections::VecDeque;
        let mut queue = VecDeque::new();
        queue.push_back(self.clone());

        while !queue.is_empty() {
            let mut next = VecDeque::new();

            for node in queue.iter() {
                match node {
                    AstNode::BinaryOp { op, left, right } => {
                        print!("{} ", op);
                        next.push_back(*left.clone());
                        next.push_back(*right.clone());
                    }
                    AstNode::Identifier(id) => print!("{} ", id),
                    AstNode::Number(n) => print!("{} ", n),
                    AstNode::Empty => print!("ε "),
                    AstNode::Error => print!("ERROR "),
                }
            }

            println!();
            queue = next;
        }
    }
}


// EXPR -> TERM EXPRDASH
pub fn parse_expr(mut tokens: VecDeque<Token>, errors: &mut Vec<ParseError>) -> (ParseTree, AstNode, VecDeque<Token>) {
    let mut node = ParseTree::new("EXPR");

    let (term_node, term_ast, tokens_after_term) = parse_term(tokens, errors);
    node.children.push(term_node);

    let (exprdash_node, expr_ast, tokens_after_exprdash) = parse_exprdash(tokens_after_term, errors, term_ast.clone());
    node.children.push(exprdash_node);

    (node, expr_ast, tokens_after_exprdash)
}

// EXPRDASH -> + TERM EXPRDASH | EPSILON 
fn parse_exprdash(mut tokens: VecDeque<Token>, errors: &mut Vec<ParseError>, left_ast: AstNode) -> (ParseTree, AstNode, VecDeque<Token>) {
    let mut node = ParseTree::new("EXPRDASH");

    match tokens.front() {
        Some(Token::Plus) => {
            tokens.pop_front();
            node.children.push(ParseTree::new("PLUS"));

            let (term_node, term_ast, tokens_after_term) = parse_term(tokens, errors);
            node.children.push(term_node);

            let combined = AstNode::BinaryOp {
                op: "+".to_string(),
                left: Box::new(left_ast),
                right: Box::new(term_ast),
            };

            let (exprdash_node, next_ast, tokens_after_exprdash) = parse_exprdash(tokens_after_term, errors, combined.clone());
            node.children.push(exprdash_node);

            (node, next_ast, tokens_after_exprdash)
        }
        Some(Token::Error(ch)) => {
            report_error(errors, &format!("Invalid character '{}' in expression", ch));
            tokens.pop_front();
            node.children.push(ParseTree::new("ERROR"));
            (node, AstNode::Error, tokens)
        }
        _ => {
            node.children.push(ParseTree::new("EPSILON"));
            (node, left_ast, tokens)
        }
    }
}

// TERM -> FACTOR TERMDASH
fn parse_term(mut tokens: VecDeque<Token>, errors: &mut Vec<ParseError>) -> (ParseTree, AstNode, VecDeque<Token>) {
    let mut node = ParseTree::new("TERM");

    let (factor_node, factor_ast, tokens_after_factor) = parse_factor(tokens, errors);
    node.children.push(factor_node);

    let (termdash_node, term_ast, tokens_after_termdash) = parse_termdash(tokens_after_factor, errors, factor_ast.clone());
    node.children.push(termdash_node);

    (node, term_ast, tokens_after_termdash)
}

// TERMDASH -> * FACTOR TERMDASH | EPSILON
fn parse_termdash(mut tokens: VecDeque<Token>, errors: &mut Vec<ParseError>, left_ast: AstNode) -> (ParseTree, AstNode, VecDeque<Token>) {
    let mut node = ParseTree::new("TERMDASH");

    match tokens.front() {
        Some(Token::Star) => {
            tokens.pop_front();
            node.children.push(ParseTree::new("STAR"));

            let (factor_node, factor_ast, tokens_after_factor) = parse_factor(tokens, errors);
            node.children.push(factor_node);

            let combined = AstNode::BinaryOp {
                op: "*".to_string(),
                left: Box::new(left_ast),
                right: Box::new(factor_ast),
            };

            let (termdash_node, term_ast, tokens_after_termdash) = parse_termdash(tokens_after_factor, errors, combined.clone());
            node.children.push(termdash_node);

            (node, term_ast, tokens_after_termdash)
        }
        Some(Token::Error(ch)) => {
            report_error(errors, &format!("Invalid character '{}' in term", ch));
            tokens.pop_front();
            node.children.push(ParseTree::new("ERROR"));
            (node, AstNode::Error, tokens)
        }
        _ => {
            node.children.push(ParseTree::new("EPSILON"));
            (node, left_ast, tokens)
        }
    }
}

// FACTOR -> IDENTIFIER | NUMBER | ( EXPR )
fn parse_factor(mut tokens: VecDeque<Token>, errors: &mut Vec<ParseError>) -> (ParseTree, AstNode, VecDeque<Token>) {
    let mut node = ParseTree::new("FACTOR");

    let next = tokens.front().cloned(); // <-- clone the front token before match
    match next {
        Some(Token::Identifier(name)) => {
            node.children.push(ParseTree::new(&format!("IDENTIFIER({})", name)));
            tokens.pop_front();
            (node, AstNode::Identifier(name), tokens)
        }
        Some(Token::Number(val)) => {
            node.children.push(ParseTree::new(&format!("NUMBER({})", val)));
            tokens.pop_front();
            (node, AstNode::Number(val), tokens)
        }
        Some(Token::BOpen) => {
            tokens.pop_front();
            node.children.push(ParseTree::new("BOPEN"));

            let (expr_node, expr_ast, tokens_after_expr) = parse_expr(tokens, errors);
            node.children.push(expr_node);

            let mut t = tokens_after_expr;
            match t.front() {
                Some(Token::BClose) => {
                    node.children.push(ParseTree::new("BCLOSE"));
                    t.pop_front();
                    (node, expr_ast, t)
                }
                other => {
                    report_error(errors, &format!("Expected ')' but found {:?}", other));
                    node.children.push(ParseTree::new("ERROR"));
                    (node, AstNode::Error, t)
                }
            }
        }
        Some(Token::BClose) => {
            report_error(errors, "Unmatched ')'");
            node.children.push(ParseTree::new("ERROR"));
            tokens.pop_front();
            (node, AstNode::Error, tokens)
        }
        Some(Token::Error(ch)) => {
            report_error(errors, &format!("Invalid character '{}' in factor", ch));
            node.children.push(ParseTree::new("ERROR"));
            tokens.pop_front();
            (node, AstNode::Error, tokens)
        }
        Some(Token::EOF) | None => {
            report_error(errors, "Unexpected end of input while parsing factor");
            node.children.push(ParseTree::new("ERROR"));
            (node, AstNode::Error, tokens)
        }
        Some(t) => {
            report_error(errors, &format!("Unexpected token in factor: {:?}", t));
            node.children.push(ParseTree::new("ERROR"));
            tokens.pop_front();
            (node, AstNode::Error, tokens)
        }
    }
}

