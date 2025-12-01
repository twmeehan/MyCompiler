use crate::parser::AstNode;
use std::collections::{HashMap, HashSet};

/// In memory representation of LLVM IR suitable for x86 backend consumption.
#[derive(Debug, Clone)]
pub enum Value {
    Imm(i64),
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

/// A binary arithmetic definition: dest = op lhs, rhs
#[derive(Debug, Clone)]
pub struct BinOp {
    pub dest: String, // e.g. "t1"
    pub op: ArithOp,
    pub lhs: Value,
    pub rhs: Value,
}

/// A load definition: dest = load i64, ptr %ptr
#[derive(Debug, Clone)]
pub struct Load {
    pub dest: String, 
    pub ptr: String,
}

#[derive(Debug, Clone)]
pub struct ICmp {
    pub dest: String,
    pub op: CmpOp,
    pub lhs: Value,
    pub rhs: Value,
}

/// A branch condition: either a literal bool or SSA name of an icmp result.
#[derive(Debug, Clone)]
pub enum Cond {
    Const(bool),
    Ssa(String),
}

/// Control-flow / side-effecting instructions in the function body.
#[derive(Debug, Clone)]
pub enum CfInstr {
    Label(String),
    Store {
        value: Value,
        ptr: String,
    },
    BrCond {
        cond: Cond,
        then_label: String,
        else_label: String,
    },
    BrUncond {
        label: String,
    },
    Ret {
        value: Value,
    },
}

//Entire function IR in a structured form.
#[derive(Debug, Clone)]
pub struct FunctionIR {
    pub name: String,
    pub args: Vec<String>,
    pub allocas: Vec<String>,
    /// Load definitions, keyed by dest SSA name (e.g. "t1")
    pub loads: HashMap<String, Load>,
    /// Arithmetic defs, keyed by dest SSA name
    pub binops: HashMap<String, BinOp>,
    /// Comparison defs, keyed by dest SSA name
    pub icmps: HashMap<String, ICmp>,
    /// Linear sequence of control-flow / side-effecting instructions and labels.
    pub cf_instrs: Vec<CfInstr>,
}

impl FunctionIR {
    /// Render the in-memory IR into a textual LLVM-like format.
    pub fn to_llvm_string(&self) -> String {
        LlvmPrinter::new(self).emit()
    }
}

/// LLVM builder that constructs an in-memory `FunctionIR`
/// that the x86 backend can consume directly.
pub struct LLVM {
    temp: usize,
    label: usize,
}

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
        let arg_list = if self.func.args.is_empty() {
            String::new()
        } else {
            self.func
                .args
                .iter()
                .map(|a| format!("i64 %{}", a))
                .collect::<Vec<_>>()
                .join(", ")
        };
        self.output
            .push_str(&format!("define i64 @{}({}) {{\n", self.func.name, arg_list));

        for instr in &self.func.cf_instrs {
            match instr {
                CfInstr::Label(name) => {
                    self.output.push_str(&format!("{}:\n", name));
                    if !self.allocas_emitted {
                        for alloc in &self.func.allocas {
                            self.push_line(&format!("%{} = alloca i64", alloc));
                        }
                        self.allocas_emitted = true;
                    }
                }
                CfInstr::Store { value, ptr } => {
                    let val = self.emit_value(value);
                    self.push_line(&format!("store i64 {}, ptr %{}", val, ptr));
                }
                CfInstr::BrUncond { label } => {
                    self.push_line(&format!("br label %{}", label));
                }
                CfInstr::BrCond {
                    cond,
                    then_label,
                    else_label,
                } => {
                    let cond_str = self.emit_cond(cond);
                    self.push_line(&format!(
                        "br i1 {}, label %{}, label %{}",
                        cond_str, then_label, else_label
                    ));
                }
                CfInstr::Ret { value } => {
                    let val = self.emit_value(value);
                    self.push_line(&format!("ret i64 {}", val));
                }
            }
        }

        self.output.push_str("}\n");
        self.output
    }

    fn push_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_value(&mut self, v: &Value) -> String {
        match v {
            Value::Imm(n) => n.to_string(),
            Value::Ssa(name) => self.ensure_value_emitted(name),
        }
    }

    fn ensure_value_emitted(&mut self, name: &str) -> String {
        if self.func.args.iter().any(|a| a == name) {
            return format!("%{}", name);
        }

        if !self.emitted_defs.contains(name) {
            if let Some(load) = self.func.loads.get(name) {
                self.push_line(&format!("%{} = load i64, ptr %{}", name, load.ptr));
            } else if let Some(bin) = self.func.binops.get(name) {
                let lhs = self.emit_value(&bin.lhs);
                let rhs = self.emit_value(&bin.rhs);
                let op = match bin.op {
                    ArithOp::Add => "add",
                    ArithOp::Sub => "sub",
                    ArithOp::Mul => "mul",
                };
                self.push_line(&format!(
                    "%{} = {} i64 {}, {}",
                    name, op, lhs, rhs
                ));
            } else if let Some(cmp) = self.func.icmps.get(name) {
                let lhs = self.emit_value(&cmp.lhs);
                let rhs = self.emit_value(&cmp.rhs);
                let op = match cmp.op {
                    CmpOp::Ult => "icmp ult",
                    CmpOp::Ugt => "icmp ugt",
                    CmpOp::Ule => "icmp ule",
                    CmpOp::Uge => "icmp uge",
                    CmpOp::Eq => "icmp eq",
                };
                self.push_line(&format!(
                    "%{} = {} i64 {}, {}",
                    name, op, lhs, rhs
                ));
            } else {
                panic!("Unknown SSA value: {}", name);
            }
            self.emitted_defs.insert(name.to_string());
        }

        format!("%{}", name)
    }

    fn emit_cond(&mut self, cond: &Cond) -> String {
        match cond {
            Cond::Const(true) => "true".to_string(),
            Cond::Const(false) => "false".to_string(),
            Cond::Ssa(name) => self.ensure_value_emitted(name),
        }
    }
}

impl LLVM {
    pub fn new() -> Self {
        LLVM { temp: 0, label: 0 }
    }

    fn new_temp(&mut self) -> String {
        self.temp += 1;
        format!("t{}", self.temp)
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let out = format!("{}.{}", prefix, self.label);
        self.label += 1;
        out
    }

    /// Generate in-memory IR (FunctionIR) instead of text.
    pub fn generate(&mut self, ast: &AstNode) -> FunctionIR {
        if let AstNode::Program { args, vars, body, ret } = ast {
            // --------- build empty FunctionIR ----------
            let mut func = FunctionIR {
                name: "foo".to_string(),
                args: args.clone(),
                allocas: vec![],
                loads: Default::default(),
                binops: Default::default(),
                icmps: Default::default(),
                cf_instrs: vec![],
            };

            // --------- entry block ----------
            func.cf_instrs.push(CfInstr::Label("entry".to_string()));

            // --------- create allocas for args ----------
            for a in args {
                func.allocas.push(format!("{}.alloc", a));

                // implicit store of arg into its alloc slot
                func.cf_instrs.push(CfInstr::Store {
                    value: Value::Ssa(a.clone()),
                    ptr: format!("{}.alloc", a),
                });
            }

            // --------- allocas for vars (no initial store) ----------
            for v in vars {
                func.allocas.push(format!("{}.alloc", v));
            }

            // --------- generate all statements ----------
            for stmt in body {
                self.gen_stmt(stmt, &mut func);
            }

            // --------- return ----------
            let ret_temp = self.new_temp();
            func.loads.insert(ret_temp.clone(), Load {
                dest: ret_temp.clone(),
                ptr: format!("{}.alloc", ret),
            });
            func.cf_instrs.push(CfInstr::Ret {
                value: Value::Ssa(ret_temp),
            });

            return func;
        }

        panic!("Expected Program AST at top");
    }

    // -------------------- Statements ------------------------

    fn gen_stmt(&mut self, stmt: &AstNode, func: &mut FunctionIR) {
        match stmt {
            AstNode::Assign { name, expr } => {
                let value = self.gen_expr(expr, func);
                func.cf_instrs.push(CfInstr::Store {
                    value,
                    ptr: format!("{}.alloc", name),
                });
            }

            AstNode::If { cond, then_body, else_body } => {
                let cond_val = self.gen_bool(cond, func);

                let l_then = self.new_label("if.then");
                let l_else = self.new_label("if.else");
                let l_end  = self.new_label("if.end");

                func.cf_instrs.push(CfInstr::BrCond {
                    cond: cond_val,
                    then_label: l_then.clone(),
                    else_label: l_else.clone(),
                });

                // then block
                func.cf_instrs.push(CfInstr::Label(l_then.clone()));
                for s in then_body {
                    self.gen_stmt(s, func);
                }
                func.cf_instrs.push(CfInstr::BrUncond { label: l_end.clone() });

                // else block
                func.cf_instrs.push(CfInstr::Label(l_else.clone()));
                for s in else_body {
                    self.gen_stmt(s, func);
                }
                func.cf_instrs.push(CfInstr::BrUncond { label: l_end.clone() });

                // end block
                func.cf_instrs.push(CfInstr::Label(l_end));
            }

            AstNode::While { cond, body } => {
                let l_cond = self.new_label("while.cond");
                let l_body = self.new_label("while.body");
                let l_end  = self.new_label("while.end");

                func.cf_instrs.push(CfInstr::BrUncond { label: l_cond.clone() });

                // cond block
                func.cf_instrs.push(CfInstr::Label(l_cond.clone()));
                let cond_val = self.gen_bool(cond, func);
                func.cf_instrs.push(CfInstr::BrCond {
                    cond: cond_val,
                    then_label: l_body.clone(),
                    else_label: l_end.clone(),
                });

                // body block
                func.cf_instrs.push(CfInstr::Label(l_body.clone()));
                for s in body {
                    self.gen_stmt(s, func);
                }
                func.cf_instrs.push(CfInstr::BrUncond { label: l_cond });

                // end
                func.cf_instrs.push(CfInstr::Label(l_end));
            }

            _ => {}
        }
    }

    // -------------------- Booleans ------------------------

    fn gen_bool(&mut self, expr: &AstNode, func: &mut FunctionIR) -> Cond {
        match expr {
            AstNode::BoolLiteral(true) => Cond::Const(true),
            AstNode::BoolLiteral(false) => Cond::Const(false),

            AstNode::BinaryOp { op, left, right } => {
                let lhs = self.gen_expr(left, func);
                let rhs = self.gen_expr(right, func);

                let dest = self.new_temp();
                let cmp_op = match op.as_str() {
                    "<"  => CmpOp::Ult,
                    ">"  => CmpOp::Ugt,
                    "<=" => CmpOp::Ule,
                    ">=" => CmpOp::Uge,
                    "==" => CmpOp::Eq,
                    _ => panic!("Invalid boolean operator"),
                };

                func.icmps.insert(dest.clone(), ICmp {
                    dest: dest.clone(),
                    op: cmp_op,
                    lhs,
                    rhs,
                });

                Cond::Ssa(dest)
            }

            _ => panic!("Invalid boolean expression"),
        }
    }

    // -------------------- Expressions ------------------------

    fn gen_expr(&mut self, expr: &AstNode, func: &mut FunctionIR) -> Value {
        match expr {
            AstNode::Number(n) => {
                let v: i64 = n.parse().unwrap();
                Value::Imm(v)
            }

            AstNode::Identifier(id) => {
                let dest = self.new_temp();
                func.loads.insert(dest.clone(), Load {
                    dest: dest.clone(),
                    ptr: format!("{}.alloc", id),
                });
                Value::Ssa(dest)
            }

            AstNode::BinaryOp { op, left, right } => {
                let lhs = self.gen_expr(left, func);
                let rhs = self.gen_expr(right, func);
                let dest = self.new_temp();

                let aop = match op.as_str() {
                    "+" => ArithOp::Add,
                    "-" => ArithOp::Sub,
                    "*" => ArithOp::Mul,
                    _ => panic!("expected arithmetic operator"),
                };

                func.binops.insert(dest.clone(), BinOp {
                    dest: dest.clone(),
                    op: aop,
                    lhs,
                    rhs,
                });

                Value::Ssa(dest)
            }

            _ => panic!("Invalid expression node"),
        }
    }
}
