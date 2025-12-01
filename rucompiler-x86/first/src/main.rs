mod scanner;
#[path = "parser_simple.rs"]
mod parser;
mod dag;
#[path = "llvm_simple.rs"]
mod llvm;
mod x86;

use scanner::{Scanner, Token};
use parser::{parse_expr, AstNode, ParseError};
use llvm::LLVM;
use x86::X86;
use std::collections::VecDeque;
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

    // Output filename for assembly
    let asm_file = Path::new(filename)
        .with_extension("s")
        .to_str()
        .unwrap()
        .to_string();

    let mut scanner = Scanner::new(&source);
    let tokens = scanner.tokenize();
    let token_queue: VecDeque<Token> = VecDeque::from(tokens);

    let mut errors: Vec<ParseError> = Vec::new();
    let (_tree, ast, remaining) = parse_expr(token_queue, &mut errors);
    let ast_invalid = matches!(&ast, AstNode::Error);
    let leftover = remaining
        .iter()
        .any(|tok| !matches!(tok, Token::EOF));

    if errors.is_empty() && !ast_invalid && !leftover {
        let mut llvm = LLVM::new();
        let func_ir = llvm.generate(&ast);

        let mut backend = X86::new();
        let mut out = File::create(&asm_file).expect("Unable to create output file");
        if let Err(err) = backend.generate(&func_ir, &mut out) {
            eprintln!("Failed to write assembly: {err}");
            std::process::exit(1);
        }
    } else {
        let mut file = File::create(&asm_file).expect("Unable to create output file");
        writeln!(file, "Parsing errors:").unwrap();
        if let Some(err) = errors.first() {
            writeln!(file, "{}", err.message).unwrap();
            eprintln!("Parsing errors:\n{}", err.message);
        } else if leftover {
            writeln!(file, "Unexpected tokens at end of input").unwrap();
            eprintln!("Parsing errors:\nUnexpected tokens at end of input");
        } else {
            writeln!(file, "Unknown parse failure").unwrap();
            eprintln!("Parsing errors:\nUnknown parse failure");
        }
    }
}
