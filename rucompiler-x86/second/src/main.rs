mod scanner;
mod parser;
mod dag;
mod llvm;
mod x86;

use scanner::Scanner;
use parser::Parser;
use llvm::LLVM;
use x86::X86;
use std::env;
use std::fs::File;
use std::io::Write;
use std::{fs, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Needs 2 arguments");
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename).expect("Could not read file");

    // Output filename for assembly (.s)
    let asm_file = Path::new(filename)
        .with_extension("s")
        .to_str()
        .unwrap()
        .to_string();

    let mut scanner = Scanner::new(&source);
    let tokens = scanner.tokenize();

    let parser = Parser::new(&tokens);
    if let Some(ast) = parser.ast {
        // Build the in-memory LLVM IR.
        let mut llvm = LLVM::new();
        let func_ir = llvm.generate(&ast);

        // Emit x86 assembly for IR
        let mut backend = X86::new();
        let mut file = File::create(&asm_file).expect("Unable to create output file");
        if let Err(err) = backend.generate(&func_ir, &mut file) {
            eprintln!("Failed to write assembly: {err}");
            std::process::exit(1);
        }
    } else {
        let mut file = File::create(&asm_file).expect("Unable to create output file");
        writeln!(file, "Parsing errors:").unwrap();
        writeln!(file, "{}", parser.errors[0]).unwrap();

        eprintln!("Parsing errors:");
        eprintln!("{}", parser.errors[0]);
    }
}
