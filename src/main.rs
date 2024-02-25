use std::io::{stdin, Write};

use evaluation::eval;
use parser::parse_expr;
use primitive_functions::load;

use crate::evaluation::create_eden_env;
use crate::lispval::LispVal::LispString;

mod evaluation;
mod parser;
mod env;
mod lispval;
mod error;
mod primitive_functions;

fn main() {
    println!("Lisp in rust!");
    let env = create_eden_env();
    load(&[LispString("/Users/izraigo/Projects/lisp/src/stdLib.scm".to_string())], &env).expect("Error");

    loop {
        let mut s = String::new();
        print!("lisp>>> ");
        std::io::stdout().flush().unwrap();
        stdin().read_line(&mut s).expect("Enter expression");
        if s.trim_end() == "quit" {
            break
        }
        match parse_expr(&s) {
            Ok((_, lisp_val)) => match eval(&lisp_val, &env) {
                Ok(res) => println!("{}", res),
                Err(e) => println!("Error: {}", e),
            },
            Err(e) => println!("Error: {}", e),
        }
    }
}

#[test]
fn eval_test() {
    let env = &create_eden_env();
    let (_, e) = parse_expr("(+ 2 \"3\")").unwrap();
    println!("Expression input: {}", e);
    let res = eval(&e, env).unwrap();
    println!("Expression input: {}", res);
    assert_eq!(res.to_string(), "5");

    let (_, e) = parse_expr("(- (+ 4 6 3) 3 5 2)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(&e, env).unwrap();
    println!("Expression input: {}", res);
    assert_eq!(res.to_string(), "3");

    let (_, e) = parse_expr("(+ 2 (- 4 1))").unwrap();
    println!("Expression input: {}", e);
    let res = eval(&e, env).unwrap();
    println!("Expression input: {}", res);
    assert_eq!(res.to_string(), "5");
}
#[test]
fn eval_test2() {
    let env = &create_eden_env();
    let (_, e) = parse_expr("(define a 2)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(&e, env).unwrap();
    println!("Expression result: {}", res);
    assert_eq!(res.to_string(), "2");

    let (_, e) = parse_expr("a").unwrap();
    println!("Expression input: {}", e);
    let res = eval(&e, env).unwrap();
    println!("Expression result: {}", res);
    assert_eq!(res.to_string(), "2");
}
