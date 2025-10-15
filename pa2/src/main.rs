mod scanner;
mod parser;
mod dag;
mod llvm;

use scanner::Scanner;
use parser::parse_expr;
use parser::ParseError;
use scanner::Token;
use parser::report_error;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::collections::VecDeque;
use dag::DagBuilder;
use llvm::LLVM;
use std::io::Write;


fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Needs 2 arguments");
        std::process::exit(1);
    }

    // read the file and get lines
    let filename =  &args[1];
    let file = File::open(filename).unwrap(); // throws error if file does not exist
    let reader = io::BufReader::new(file);

    // for each line run scanner and then parse
    for line in reader.lines() {

        let line = line.unwrap(); // throws error if line is not valid
        if line.trim().is_empty() { 
            continue; 
        }
        let mut scanner = Scanner::new(&line);
        let tokens = scanner.tokenize();

        let queue = VecDeque::from(tokens);

        let mut errors: Vec<ParseError> = Vec::new();

        let (tree, ast, mut remaining) = parse_expr(queue, &mut errors);

        if !remaining.is_empty() && !matches!(remaining.front(), Some(Token::EOF)) {
            report_error(&mut errors, "Extra or unmatched tokens after valid expression");
        }
        if !errors.is_empty() {
            println!("\nErrors encountered:");
            for e in &errors {
                println!("- {}", e.message);
            }
            let mut file = File::create("first.ll").expect("Failed to create LLVM file");
            writeln!(file, "; Unable to parse input").unwrap();
        } else {
            println!("AST");
            ast.print();
            let mut builder = DagBuilder::new();
            let root_id = builder.from_ast(&ast);
            println!("DAG");
            builder.print(root_id);
            let mut llvm = LLVM::new();
            llvm.generate(&builder, root_id, "first.ll");
        }
        println!();
    }
}
