use crate::parser::AstNode;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct DagNode {
    pub id: usize,
    pub label: String,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

#[derive(Default)]
pub struct DagBuilder {
    pub nodes: Vec<DagNode>,
    pub map: HashMap<String, usize>,
    next_id: usize,
}

impl DagBuilder {
    pub fn new() -> Self {
        DagBuilder::default()
    }

    /// Walk program and build DAG only for expressions.
    pub fn from_ast(&mut self, ast: &AstNode) {
        match ast {
            AstNode::Program { body, .. } => {
                for stmt in body {
                    self.from_ast(stmt);
                }
            }

            AstNode::Assign { expr, .. } => {
                self.from_expr(expr);
            }

            AstNode::If { cond, then_body, else_body } => {
                self.from_expr(cond);
                for stmt in then_body { self.from_ast(stmt); }
                for stmt in else_body { self.from_ast(stmt); }
            }

            AstNode::While { cond, body } => {
                self.from_expr(cond);
                for stmt in body { self.from_ast(stmt); }
            }

            AstNode::BinaryOp { .. }
            | AstNode::Identifier(_)
            | AstNode::Number(_)
            | AstNode::BoolLiteral(_) => {
                self.from_expr(ast);
            }

            AstNode::Empty | AstNode::Error => {}
        }
    }

    /// Build DAG for expressions only.
    pub fn from_expr(&mut self, ast: &AstNode) -> usize {
        match ast {
            AstNode::Identifier(id) => self.make_leaf(id),
            AstNode::Number(n) => self.make_leaf(n),
            AstNode::BoolLiteral(b) => self.make_leaf(&b.to_string()),

            AstNode::BinaryOp { op, left, right } => {
                let l_id = self.from_expr(left);
                let r_id = self.from_expr(right);
                let key = format!("{}({},{})", op, l_id, r_id);

                if let Some(&id) = self.map.get(&key) {
                    return id; // common subexpression reuse
                }

                let id = self.next_id;
                self.next_id += 1;
                self.nodes.push(DagNode {
                    id,
                    label: op.clone(),
                    left: Some(l_id),
                    right: Some(r_id),
                });
                self.map.insert(key, id);
                id
            }

            AstNode::Empty => self.make_leaf("ε"),
            AstNode::Error => self.make_leaf("ERR"),

            _ => panic!("from_expr called on non-expression node: {:?}", ast),
        }
    }

    fn make_leaf(&mut self, label: &str) -> usize {
        let is_constant =
            label.chars().all(|c| c.is_ascii_digit()) ||
            label == "true" || label == "false" ||
            label == "ε" || label == "ERR";

        if is_constant {
            if let Some(&id) = self.map.get(label) {
                return id;
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        self.nodes.push(DagNode {
            id,
            label: label.to_string(),
            left: None,
            right: None,
        });

        if is_constant {
            self.map.insert(label.to_string(), id);
        }

        id
    }

    pub fn dump(&self) {
        println!("===== DAG Nodes =====");
        for node in &self.nodes {
            println!(
                "id: {}, label: {}, left: {:?}, right: {:?}",
                node.id, node.label, node.left, node.right
            );
        }
    }

    pub fn print(&self, root_id: usize) {
        use std::collections::{HashSet, VecDeque};

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(root_id);
        visited.insert(root_id);

        while !queue.is_empty() {
            let mut next = VecDeque::new();

            for &id in &queue {
                let node = &self.nodes[id];
                print!("{} ", node.label);

                if let Some(l) = node.left {
                    if visited.insert(l) {
                        next.push_back(l);
                    }
                }
                if let Some(r) = node.right {
                    if visited.insert(r) {
                        next.push_back(r);
                    }
                }
            }

            println!();
            queue = next;
        }
    }
}
