use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use crate::error::LispErr;
use crate::error::LispErr::Runtime;
use crate::evaluation::eval;
use crate::lispval::LispVal;
use crate::parser::parse_expr_list;
use std::string::String;
use crate::env::Env;

pub fn load(file_path: &str, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut file = match File::open(Path::new(file_path)) {
        Err(_) => return Err(Runtime("Error reading file".to_string())),
        Ok(f) => f,
    };
    let mut s = String::new();
    file.read_to_string(&mut s);
    let expressions = match parse_expr_list(&s) {
        Ok((_, v)) => v,
        Err(err) => return Err(Runtime(err.to_string())),
    };

    if expressions.len() == 0 {
        return Ok(LispVal::List(vec![]));
    }

    expressions[1..].iter()
        .try_fold(eval(&expressions[0], env)?, |_, v| eval(v, env))
}