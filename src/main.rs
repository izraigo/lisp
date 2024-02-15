use std::cell::RefCell;
use std::rc::Rc;
use std::io::{stdin, Write};

use evaluation::eval;
use parser::parse_expr;

use crate::env::Closure;

mod evaluation;
mod parser;
mod env;
mod lispval;
mod lispErr;

fn main() {
    println!("Lisp in rust!");
    let env = Rc::from(RefCell::new(Closure::new()));
    loop {
        let mut s = String::new();
        print!("lisp>>> ");
        std::io::stdout().flush().unwrap();
        stdin().read_line(&mut s).expect("Enter expression");

        match parse_expr(&s) {
            Ok((_, lisp_val)) => match eval(lisp_val, env.clone()) {
                Ok(res) => println!("{}", res),
                Err(e) => println!("Error: {}", e),
            },
            Err(e) => println!("Error: {}", e),
        }
    }
}

#[test]
fn eval_test() {
    let env = Rc::from(RefCell::new(Closure::new()));
    let (_, e) = parse_expr("(+ 2 \"3\")").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env.clone()).unwrap();
    println!("Expression input: {}", res);
    assert_eq!(res.to_string(), "5");

    let (_, e) = parse_expr("(- (+ 4 6 3) 3 5 2)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env.clone()).unwrap();
    println!("Expression input: {}", res);
    assert_ne!(res.to_string(), "5");

    let (_, e) = parse_expr("(+ 2 (- 4 1))").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env).unwrap();
    println!("Expression input: {}", res);
    assert_eq!(res.to_string(), "5");
}
#[test]
fn eval_test2() {
    let env = Rc::from(RefCell::new(Closure::new()));
    let (_, e) = parse_expr("(define a 2)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env.clone()).unwrap();
    println!("Expression result: {}", res);
    assert_eq!(res.to_string(), "2");

    let (_, e) = parse_expr("(a)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env).unwrap();
    println!("Expression result: {}", res);
    assert_eq!(res.to_string(), "2");
}
