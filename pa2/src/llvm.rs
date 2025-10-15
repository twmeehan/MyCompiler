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
        // 1. collect identifiers for function args
        let args = Self::collect_identifiers(dag, root);
        let arg_list = args
            .iter()
            .map(|id| format!("i64 %{}", id))
            .collect::<Vec<_>>()
            .join(", ");

        // 2. recursively emit IR for DAG
        let root_val = self.emit_node(dag, root);

        // 3. build LLVM function text
        let mut output = Vec::new();
        output.push(format!("define i64 @foo({}) {{", arg_list));
        for line in &self.code {
            output.push(format!("    {}", line));
        }
        output.push(format!("    ret i64 {}", root_val));
        output.push("}".to_string());

        // 4. write to file
        let mut file = File::create(filename).expect("Unable to create output file");
        for line in &output {
            writeln!(file, "{}", line).unwrap();
        }
    }

    fn collect_identifiers(dag: &DagBuilder, root: usize) -> Vec<String> {
        let mut stack = vec![root];
        let mut ids = HashSet::new();

        while let Some(id) = stack.pop() {
            let node = &dag.nodes[id];
            if let Some(l) = node.left {
                stack.push(l);
            }
            if let Some(r) = node.right {
                stack.push(r);
            }

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

    fn emit_node(&mut self, dag: &DagBuilder, id: usize) -> String {
        // if we've already assigned a temp, reuse it
        if let Some(temp) = self.temp_map.get(&id) {
            return temp.clone();
        }

        let node = &dag.nodes[id];
        // Leaf node
        if node.left.is_none() && node.right.is_none() {
            if node.label.chars().all(|c| c.is_ascii_alphabetic()) {
                return format!("%{}", node.label);
            } else {
                return node.label.clone(); // number literal
            }
        }

        // Recurse on children
        let left_val = self.emit_node(dag, node.left.unwrap());
        let right_val = self.emit_node(dag, node.right.unwrap());

        // Create new temp for this computation
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
