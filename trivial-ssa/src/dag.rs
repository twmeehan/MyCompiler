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

    // get DAG from AST
    pub fn from_ast(&mut self, ast: &AstNode) -> usize {
        match ast {
            AstNode::Identifier(id) => self.make_leaf(id),
            AstNode::Number(n) => self.make_leaf(n),
            AstNode::BinaryOp { op, left, right } => {
                let l_id = self.from_ast(left);
                let r_id = self.from_ast(right);
                let key = format!("{}({},{})", op, l_id, r_id);

                if let Some(&id) = self.map.get(&key) {
                    return id; // common subexpression found
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
        }
    }

    fn make_leaf(&mut self, label: &str) -> usize {
        if let Some(&id) = self.map.get(label) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(DagNode {
            id,
            label: label.to_string(),
            left: None,
            right: None,
        });
        self.map.insert(label.to_string(), id);
        id
    }

    // follow bst
    pub fn print(&self, root_id: usize) {
        use std::collections::{HashSet, VecDeque};
    
        let mut queue = VecDeque::new();
        // keep track of visited so we dont print duplicates
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
