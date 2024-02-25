use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::string::String;

use crate::env::Env;
use crate::error::LispErr;
use crate::error::LispErr::Runtime;
use crate::evaluation::{eval, unpack_str};
use crate::lispval::LispVal;
use crate::parser::parse_vector;

pub fn load(a: &[LispVal], env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if a.len() != 1 {
        return Err(Runtime("Expected one argument".to_string()));
    }

    let mut file = match File::open(Path::new(&unpack_str(&a[0])?)) {
        Err(_) => return Err(Runtime("Error reading file".to_string())),
        Ok(f) => f,
    };
    let mut s = String::new();
    _ = file.read_to_string(&mut s);
    let expressions = match parse_vector(&s) {
        Ok((_, v)) => v,
        Err(err) => return Err(Runtime(err.to_string())),
    };

    //expressions.iter().for_each(|e| println!("{}", e));

    match expressions.len() {
        0 => return Ok(LispVal::List(vec![])),
        1 => eval(&expressions[0], &env),
        _ => expressions[1..].iter().try_fold(eval(&expressions[0], &env)?, |_, v| eval(v, &env))
    }
}