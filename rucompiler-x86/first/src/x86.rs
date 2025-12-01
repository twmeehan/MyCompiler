use crate::llvm::{
    ArithOp, BinOp, CfInstr, CmpOp, Cond, FunctionIR, Value,
};
use std::collections::HashMap;
use std::io::{self, Write};

/// X86 code generator: takes a FunctionIR and emits x86-64 assembly.
pub struct X86;

impl X86 {
    pub fn new() -> Self {
        X86
    }

    /// Emit x86-64 assembly for the given function IR into `out`.
    pub fn generate<W: Write>(&mut self, func: &FunctionIR, out: &mut W) -> io::Result<()> {
        emit_function(func, out)
    }
}

fn emit_function<W: Write>(func: &FunctionIR, out: &mut W) -> io::Result<()> {
    // Map each alloca to a stack offset: -8, -16, -24, ...
    let mut alloc_offsets: HashMap<String, i32> = HashMap::new();
    for (i, name) in func.allocas.iter().enumerate() {
        let offset = -8 * (i as i32 + 1);
        alloc_offsets.insert(name.clone(), offset);
    }
    let stack_size = (func.allocas.len() as i32) * 8;

    // Map arg names to hardware argument registers.
    let hw_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
    let mut arg_regs: HashMap<String, &'static str> = HashMap::new();
    for (i, arg_name) in func.args.iter().enumerate() {
        if i < hw_regs.len() {
            arg_regs.insert(arg_name.clone(), hw_regs[i]);
        }
    }

    // Prologue
    writeln!(out, ".text")?;
    writeln!(out, ".global {}", func.name)?;
    writeln!(out, "{}:", func.name)?;
    writeln!(out, "    pushq %rbp")?;
    writeln!(out, "    movq %rsp, %rbp")?;
    writeln!(out, "    pushq %rbx")?;
    if stack_size > 0 {
        writeln!(out, "    subq ${}, %rsp", stack_size)?;
    }

    // Body
    for instr in &func.cf_instrs {
        match instr {
            CfInstr::Label(name) => {
                if name != "entry" {
                    writeln!(out, "{}:", name)?;
                }
            }
            CfInstr::Store { value, ptr } => {
                emit_store(value, ptr, func, &alloc_offsets, &arg_regs, out)?;
            }
            CfInstr::BrUncond { label } => {
                writeln!(out, "    jmp {}", label)?;
            }
            CfInstr::BrCond {
                cond,
                then_label,
                else_label,
            } => {
                emit_cond_br(cond, then_label, else_label, func, &alloc_offsets, &arg_regs, out)?;
            }
            CfInstr::Ret { value } => {
                // Evaluate return value into %rax
                eval_value(value, func, &alloc_offsets, &arg_regs, out)?;
                // Epilogue
                if stack_size > 0 {
                    writeln!(out, "    addq ${}, %rsp", stack_size)?;
                }
                writeln!(out, "    popq %rbx")?;
                writeln!(out, "    popq %rbp")?;
                writeln!(out, "    ret")?;
            }
        }
    }

    // GNU stack note
    writeln!(out, ".section .note.GNU-stack,\"\",@progbits")?;

    Ok(())
}

// store i64 <value>, ptr %ptr
fn emit_store<W: Write>(
    value: &Value,
    ptr: &str,
    func: &FunctionIR,
    alloc_offsets: &HashMap<String, i32>,
    arg_regs: &HashMap<String, &'static str>,
    out: &mut W,
) -> io::Result<()> {
    let offset = *alloc_offsets
        .get(ptr)
        .unwrap_or_else(|| panic!("unknown alloca ptr: {}", ptr));

    match value {
        // store i64 <imm>, ptr %x.alloc  ->  movq $imm, offset(%rbp)
        Value::Imm(n) => {
            writeln!(out, "    movq ${}, {}(%rbp)", n, offset)?;
        }
        // store i64 %a, ptr %a.alloc -> movq %rdi, offset(%rbp)
        Value::Ssa(name) if arg_regs.contains_key(name) => {
            let src_reg = arg_regs[name];
            writeln!(out, "    movq {}, {}(%rbp)", src_reg, offset)?;
        }
        // general case: eval into %rax then store
        _ => {
            eval_value(value, func, alloc_offsets, arg_regs, out)?;
            writeln!(out, "    movq %rax, {}(%rbp)", offset)?;
        }
    }

    Ok(())
}

/// Evaluate a Value so that the result is in %rax.
fn eval_value<W: Write>(
    v: &Value,
    func: &FunctionIR,
    alloc_offsets: &HashMap<String, i32>,
    arg_regs: &HashMap<String, &'static str>,
    out: &mut W,
) -> io::Result<()> {
    match v {
        Value::Imm(n) => {
            writeln!(out, "    movq ${}, %rax", n)?;
        }
        Value::Ssa(name) => {
            // Argument value?
            if let Some(&reg) = arg_regs.get(name) {
                writeln!(out, "    movq {}, %rax", reg)?;
                return Ok(());
            }

            // Load result?
            if let Some(load) = func.loads.get(name) {
                let offset = *alloc_offsets
                    .get(&load.ptr)
                    .unwrap_or_else(|| panic!("unknown alloca ptr: {}", load.ptr));
                writeln!(out, "    movq {}(%rbp), %rax", offset)?;
                return Ok(());
            }

            // Arithmetic result?
            if let Some(bin) = func.binops.get(name) {
                eval_binop(bin, func, alloc_offsets, arg_regs, out)?;
                return Ok(());
            }

            panic!("Unknown SSA value: {}", name);
        }
    }
    Ok(())
}

fn eval_binop<W: Write>(
    bin: &BinOp,
    func: &FunctionIR,
    alloc_offsets: &HashMap<String, i32>,
    arg_regs: &HashMap<String, &'static str>,
    out: &mut W,
) -> io::Result<()> {
    // Evaluate rhs -> %rax, push;
    eval_value(&bin.rhs, func, alloc_offsets, arg_regs, out)?;
    writeln!(out, "    pushq %rax")?;
    // Evaluate lhs -> %rax
    eval_value(&bin.lhs, func, alloc_offsets, arg_regs, out)?;
    // Pop rhs into %rbx
    writeln!(out, "    popq %rbx")?;

    match bin.op {
        ArithOp::Add => {
            writeln!(out, "    addq %rbx, %rax")?;
        }
        ArithOp::Sub => {
            // lhs - rhs  (rax = lhs, rbx = rhs)
            writeln!(out, "    subq %rbx, %rax")?;
        }
        ArithOp::Mul => {
            writeln!(out, "    imulq %rbx, %rax")?;
        }
    }
    Ok(())
}

fn eval_value_into_reg<W: Write>(
    v: &Value,
    target: &str,
    func: &FunctionIR,
    alloc_offsets: &HashMap<String, i32>,
    arg_regs: &HashMap<String, &'static str>,
    out: &mut W,
) -> io::Result<()> {
    match v {
        Value::Imm(n) => {
            writeln!(out, "    movq ${}, {}", n, target)?;
            return Ok(());
        }
        Value::Ssa(name) => {
            // Argument?
            if let Some(&reg) = arg_regs.get(name) {
                writeln!(out, "    movq {}, {}", reg, target)?;
                return Ok(());
            }
            // Simple load %t = load i64, ptr %x.alloc
            if let Some(load) = func.loads.get(name) {
                let offset = *alloc_offsets
                    .get(&load.ptr)
                    .unwrap_or_else(|| panic!("unknown alloca ptr: {}", load.ptr));
                writeln!(out, "    movq {}(%rbp), {}", offset, target)?;
                return Ok(());
            }
        }
    }

    // General case: compute into %rax then move.
    eval_value(v, func, alloc_offsets, arg_regs, out)?;
    if target != "%rax" {
        writeln!(out, "    movq %rax, {}", target)?;
    }
    Ok(())
}

// Conditional branch
fn emit_cond_br<W: Write>(
    cond: &Cond,
    then_label: &str,
    else_label: &str,
    func: &FunctionIR,
    alloc_offsets: &HashMap<String, i32>,
    arg_regs: &HashMap<String, &'static str>,
    out: &mut W,
) -> io::Result<()> {
    match cond {
        Cond::Const(true) => {
            writeln!(out, "    jmp {}", then_label)?;
        }
        Cond::Const(false) => {
            writeln!(out, "    jmp {}", else_label)?;
        }
        Cond::Ssa(name) => {
            let cmp = func
                .icmps
                .get(name)
                .unwrap_or_else(|| panic!("branch on non-icmp SSA {}", name));

            if let Value::Imm(immediate) = cmp.rhs {
                eval_value_into_reg(&cmp.lhs, "%rbx", func, alloc_offsets, arg_regs, out)?;
                writeln!(out, "    cmpq ${}, %rbx", immediate)?;

                match cmp.op {
                    CmpOp::Eq => {
                        writeln!(out, "    je {}", then_label)?;
                        writeln!(out, "    jmp {}", else_label)?;
                    }
                    CmpOp::Ult => {
                        // lhs < rhs  (unsigned below)
                        writeln!(out, "    jb {}", then_label)?;
                        writeln!(out, "    jmp {}", else_label)?;
                    }
                    CmpOp::Ugt => {
                        // lhs > rhs  (unsigned above)
                        writeln!(out, "    ja {}", then_label)?;
                        writeln!(out, "    jmp {}", else_label)?;
                    }
                    CmpOp::Ule => {
                        // lhs <= rhs  <=> not (lhs > rhs)
                        writeln!(out, "    ja {}", else_label)?;
                        writeln!(out, "    jmp {}", then_label)?;
                    }
                    CmpOp::Uge => {
                        // lhs >= rhs  <=> not (lhs < rhs)
                        writeln!(out, "    jb {}", else_label)?;
                        writeln!(out, "    jmp {}", then_label)?;
                    }
                }
                return Ok(());
            }

            // Fallback: generic compare using %rax/%rbx and stack, as before.
            eval_value(&cmp.rhs, func, alloc_offsets, arg_regs, out)?;
            writeln!(out, "    pushq %rax")?;
            eval_value(&cmp.lhs, func, alloc_offsets, arg_regs, out)?;
            writeln!(out, "    popq %rbx")?;
            writeln!(out, "    cmpq %rbx, %rax")?;

            match cmp.op {
                CmpOp::Eq => {
                    writeln!(out, "    je {}", then_label)?;
                    writeln!(out, "    jmp {}", else_label)?;
                }
                CmpOp::Ult => {
                    writeln!(out, "    jb {}", then_label)?;
                    writeln!(out, "    jmp {}", else_label)?;
                }
                CmpOp::Ugt => {
                    writeln!(out, "    ja {}", then_label)?;
                    writeln!(out, "    jmp {}", else_label)?;
                }
                CmpOp::Ule => {
                    writeln!(out, "    ja {}", else_label)?;
                    writeln!(out, "    jmp {}", then_label)?;
                }
                CmpOp::Uge => {
                    writeln!(out, "    jb {}", else_label)?;
                    writeln!(out, "    jmp {}", then_label)?;
                }
            }
        }
    }
    Ok(())
}
