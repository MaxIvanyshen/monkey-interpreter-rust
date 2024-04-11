use std::io::{self, Write};
use lexer::Lexer;
use std::rc::Rc;
use std::cell::RefCell;
use parser::Parser;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        run_file(&args[1]);
    } else {
        repl();
    }
}

fn repl() {
    let msg = "This is monkey programming language!\nFeel free to type in commands";
    let prompt = ">> ";
    println!("{}", msg);
    let environment = Rc::new(RefCell::new(object::Environment::new()));
    loop {
        print!("{}", prompt);
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input).unwrap();

        let l = Lexer::new(&input);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        if p.errors().len() != 0 {
            println!(" parser errors:");
            for msg in p.errors() {
                println!("\t{}", msg);
            }
            continue;
        }
        println!("{}", evaluator::evaluate_program(program, environment.clone()).unwrap().inspect());
    }
}

fn run_file(filename: &str) {
    let input = std::fs::read_to_string(filename).unwrap();
    let l = Lexer::new(&input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    if p.errors().len() != 0 {
        println!(" parser errors:");
        for msg in p.errors() {
            println!("\t{}", msg);
        }
        return;
    }
    let environment = Rc::new(RefCell::new(object::Environment::new()));
    println!("{}", evaluator::evaluate_program(program, environment).unwrap().inspect());
}
