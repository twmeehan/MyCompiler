use crate::parser::AstNode;
use std::collections::{HashMap, HashSet};

/// ---------- Value & Ops ----------

#[derive(Debug, Clone)]
pub enum Value {
    Imm(i64),
    /// SSA name *without* '%' (e.g., "a", "t1")
    Ssa(String),
}

#[derive(Debug, Clone, Copy)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug, Clone, Copy)]
pub enum CmpOp {
    Ult, // unsigned <
    Ugt, // unsigned >
    Ule, // unsigned <=
    Uge, // unsigned >=
    Eq,  // ==
}

/// ---------- Defs ----------

#[derive(Debug, Clone)]
pub struct BinOp {
    pub dest: String,
    pub op: ArithOp,
    pub lhs: Value,
    pub rhs: Value,
}

#[derive(Debug, Clone)]
pub struct Load {
    pub dest: String,
    /// alloca name (no leading '%'), e.g. "a.alloc"
    pub ptr: String,
}

#[derive(Debug, Clone)]
pub struct ICmp {
    pub dest: String,
    pub op: CmpOp,
    pub lhs: Value,
    pub rhs: Value,
}

/// ---------- Control Flow ----------

#[derive(Debug, Clone)]
pub enum Cond {
    Const(bool),
    /// SSA name of an icmp (no leading '%')
    Ssa(String),
}

#[derive(Debug, Clone)]
pub enum CfInstr {
    Label(String),
    Store { value: Value, ptr: String },
    BrCond { cond: Cond, then_label: String, else_label: String },
    BrUncond { label: String },
    Ret { value: Value },
}

/// ---------- Function IR ----------

#[derive(Debug, Clone)]
pub struct FunctionIR {
    pub name: String,
    pub args: Vec<String>,
    pub allocas: Vec<String>,
    pub loads: HashMap<String, Load>,
    pub binops: HashMap<String, BinOp>,
    pub icmps: HashMap<String, ICmp>,
    pub cf_instrs: Vec<CfInstr>,
}

impl FunctionIR {
    pub fn to_llvm_string(&self) -> String {
        LlvmPrinter::new(self).emit()
    }
}

/// ---------- LLVM module builder (AST -> FunctionIR) ----------

pub struct LLVM {
    temp: usize,
}

impl LLVM {
    pub fn new() -> Self {
        LLVM { temp: 0 }
    }

    fn new_temp(&mut self) -> String {
        self.temp += 1;
        format!("t{}", self.temp)
    }

    /// Build in-memory IR for the whole function from the expression AST.
    pub fn generate(&mut self, ast: &AstNode) -> FunctionIR {
        let args = collect_identifiers(ast);
        let mut func = FunctionIR {
            name: "foo".to_string(),
            args,
            allocas: vec![],
            loads: Default::default(),
            binops: Default::default(),
            icmps: Default::default(),
            cf_instrs: vec![CfInstr::Label("entry".to_string())],
        };

        let value = self.emit_expr(ast, &mut func);
        func.cf_instrs.push(CfInstr::Ret { value });
        func
    }

    fn emit_expr(&mut self, expr: &AstNode, func: &mut FunctionIR) -> Value {
        match expr {
            AstNode::Number(n) => Value::Imm(n.parse::<i64>().unwrap()),
            AstNode::Identifier(id) => Value::Ssa(id.clone()),
            AstNode::BinaryOp { op, left, right } => {
                let lhs = self.emit_expr(left, func);
                let rhs = self.emit_expr(right, func);
                let temp = self.new_temp();
                let arith = match op.as_str() {
                    "+" => ArithOp::Add,
                    "-" => ArithOp::Sub,
                    "*" => ArithOp::Mul,
                    _ => panic!("Unsupported operator {}", op),
                };
                func.binops.insert(
                    temp.clone(),
                    BinOp {
                        dest: temp.clone(),
                        op: arith,
                        lhs,
                        rhs,
                    },
                );
                Value::Ssa(temp)
            }
            AstNode::Empty => Value::Imm(0),
            AstNode::Error => panic!("Cannot lower erroneous AST"),
        }
    }
}

fn collect_identifiers(ast: &AstNode) -> Vec<String> {
    fn visit(node: &AstNode, seen: &mut HashSet<String>, order: &mut Vec<String>) {
        match node {
            AstNode::Identifier(name) => {
                if seen.insert(name.clone()) {
                    order.push(name.clone());
                }
            }
            AstNode::BinaryOp { left, right, .. } => {
                visit(left, seen, order);
                visit(right, seen, order);
            }
            _ => {}
        }
    }

    let mut set = HashSet::new();
    let mut order = Vec::new();
    visit(ast, &mut set, &mut order);
    order
}

/// ---------- Textual LLVM-like printer (FunctionIR -> String) ----------

struct LlvmPrinter<'a> {
    func: &'a FunctionIR,
    emitted_defs: HashSet<String>,
    output: String,
    allocas_emitted: bool,
}

impl<'a> LlvmPrinter<'a> {
    fn new(func: &'a FunctionIR) -> Self {
        Self {
            func,
            emitted_defs: HashSet::new(),
            output: String::new(),
            allocas_emitted: false,
        }
    }

    fn emit(mut self) -> String {
        let sig = if self.func.args.is_empty() {
            String::new()
        } else {
            self.func.args.iter().map(|a| format!("i64 %{}", a)).collect::<Vec<_>>().join(", ")
        };
        self.output.push_str(&format!("define i64 @{}({}) {{\n", self.func.name, sig));

        for instr in &self.func.cf_instrs {
            match instr {
                CfInstr::Label(name) => {
                    self.output.push_str(&format!("{}:\n", name));
                    if !self.allocas_emitted {
                        for a in &self.func.allocas {
                            self.push_line(&format!("%{} = alloca i64", a));
                        }
                        self.allocas_emitted = true;
                    }
                }
                CfInstr::Store { value, ptr } => {
                    let v = self.emit_value(value);
                    self.push_line(&format!("store i64 {}, ptr %{}", v, ptr));
                }
                CfInstr::BrUncond { label } => {
                    self.push_line(&format!("br label %{}", label));
                }
                CfInstr::BrCond { cond, then_label, else_label } => {
                    let c = self.emit_cond(cond);
                    self.push_line(&format!("br i1 {}, label %{}, label %{}", c, then_label, else_label));
                }
                CfInstr::Ret { value } => {
                    let v = self.emit_value(value);
                    self.push_line(&format!("ret i64 {}", v));
                }
            }
        }

        self.output.push_str("}\n");
        self.output
    }

    fn push_line(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn emit_value(&mut self, v: &Value) -> String {
        match v {
            Value::Imm(n) => n.to_string(),
            Value::Ssa(name) => self.ensure_value(name),
        }
    }

    fn emit_cond(&mut self, c: &Cond) -> String {
        match c {
            Cond::Const(true) => "true".into(),
            Cond::Const(false) => "false".into(),
            Cond::Ssa(name) => self.ensure_value(name),
        }
    }

    fn ensure_value(&mut self, name: &str) -> String {
        if self.func.args.iter().any(|a| a == name) {
            return format!("%{}", name);
        }
        if !self.emitted_defs.contains(name) {
            if let Some(ld) = self.func.loads.get(name) {
                self.push_line(&format!("%{} = load i64, ptr %{}", name, ld.ptr));
            } else if let Some(b) = self.func.binops.get(name) {
                let lhs = self.emit_value(&b.lhs);
                let rhs = self.emit_value(&b.rhs);
                let op = match b.op { ArithOp::Add => "add", ArithOp::Sub => "sub", ArithOp::Mul => "mul" };
                self.push_line(&format!("%{} = {} i64 {}, {}", name, op, lhs, rhs));
            } else if let Some(cmp) = self.func.icmps.get(name) {
                let lhs = self.emit_value(&cmp.lhs);
                let rhs = self.emit_value(&cmp.rhs);
                let pred = match cmp.op {
                    CmpOp::Ult => "icmp ult",
                    CmpOp::Ugt => "icmp ugt",
                    CmpOp::Ule => "icmp ule",
                    CmpOp::Uge => "icmp uge",
                    CmpOp::Eq  => "icmp eq",
                };
                self.push_line(&format!("%{} = {} i64 {}, {}", name, pred, lhs, rhs));
            } else {
                panic!("unknown SSA {:?}", name);
            }
            self.emitted_defs.insert(name.to_string());
        }
        format!("%{}", name)
    }
}
