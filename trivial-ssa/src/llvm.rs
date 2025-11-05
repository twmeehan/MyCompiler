use crate::dag::{DagBuilder, DagNode};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;

pub struct LLVM {
    temp_counter: usize,
    temp_map: HashMap<usize, String>,
    code: Vec<String>,
}

impl LLVM {
    pub fn new() -> Self {
        Self {
            temp_counter: 1,
            temp_map: HashMap::new(),
            code: Vec::new(),
        }
    }

    pub fn generate(&mut self, dag: &DagBuilder, root: usize, filename: &str) {
        let args = Self::collect_identifiers(dag, root);
        let arg_list = args
            .iter()
            .map(|id| format!("i64 %{}", id))
            .collect::<Vec<_>>()
            .join(", ");

        let root_val = self.emit_node(dag, root);

        let mut output = Vec::new();
        output.push(format!("define i64 @foo({}) {{", arg_list));
        for line in &self.code {
            output.push(format!("    {}", line));
        }
        output.push(format!("    ret i64 {}", root_val));
        output.push("}".to_string());

        let mut file = File::create(filename).expect("Unable to create output file");
        for line in &output {
            writeln!(file, "{}", line).unwrap();
        }
    }

    fn collect_identifiers(dag: &DagBuilder, root: usize) -> Vec<String> {
        // put dag in a stack
        let mut stack = vec![root];
        let mut ids = HashSet::new();

        // grab a node off the stack
        while let Some(id) = stack.pop() {
            let node = &dag.nodes[id];
            if let Some(l) = node.left {
                stack.push(l);
            }
            if let Some(r) = node.right {
                stack.push(r);
            }

            // A leaf node
            if node.left.is_none() && node.right.is_none() {
                if node.label.chars().all(|c| c.is_ascii_alphabetic()) {
                    ids.insert(node.label.clone());
                }
            }
        }

        let mut v: Vec<_> = ids.into_iter().collect();
        v.sort();
        v
    }
    
    //Emit LLVM code recursively for a node
    fn emit_node(&mut self, dag: &DagBuilder, id: usize) -> String {
        if let Some(temp) = self.temp_map.get(&id) {
            return temp.clone();
        }

        let node = &dag.nodes[id];
        if node.left.is_none() && node.right.is_none() {
            if node.label.chars().all(|c| c.is_ascii_alphabetic()) {
                return format!("%{}", node.label);
            } else {
                return node.label.clone();
            }
        }
        let left_val = self.emit_node(dag, node.left.unwrap());
        let right_val = self.emit_node(dag, node.right.unwrap());
        let temp = format!("%t{}", self.temp_counter);
        self.temp_counter += 1;

        let op = match node.label.as_str() {
            "+" => "add",
            "*" => "mul",
            _ => "unknown",
        };

        self.code
            .push(format!("{} = {} i64 {}, {}", temp, op, left_val, right_val));
        self.temp_map.insert(id, temp.clone());
        temp
    }
}
