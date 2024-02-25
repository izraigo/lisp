use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::string::String;

use crate::env::Env;
use crate::error::LispErr;
use crate::error::LispErr::Runtime;
use crate::evaluation::eval;
use crate::lispval::LispVal;
use crate::lispval::LispVal::{Boolean, DottedList, List, Number, PrimitiveFunc};
use crate::parser::parse_vector;

pub fn load(a: &[LispVal], env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if a.len() != 1 {
        return Err(Runtime("Expected one argument".to_string()));
    }

    let mut file = match File::open(Path::new(&a[0].str()?)) {
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
fn cons(p: &[LispVal], _: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if p.len() != 2 {
        return Err(Runtime("Expected two arguments".to_string()));
    }
    let a = &p[0];
    let b = &p[1];
    match b {
        LispVal::List(xs) =>
            match xs.len() {
                0 => Ok(List(vec![a.clone()])),
                _ => {
                    let mut list = xs.clone();
                    list.insert(0, a.clone());
                    Ok(List(list))
                },
            },
        LispVal::DottedList(xs, r) => {
            let mut list = xs.clone();
            list.insert(0, a.clone());
            Ok(DottedList(list, r.clone()))
        },
        _ => Ok(DottedList(vec![a.clone()], Box::new(b.clone())))
    }
}

fn car(a: &[LispVal], _: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if a.len() != 1 {
        return Err(Runtime("Expected one argument".to_string()));
    }
    match &a[0] {
        LispVal::List(v) => Ok(v[0].clone()),
        LispVal::DottedList(v, _) => Ok(v[0].clone()),
        _ => Err(Runtime("Expected list".to_string()))
    }
}

fn cdr(a: &[LispVal], _: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if a.len() > 1 {
        return Err(Runtime("Expected one argument".to_string()));
    }
    match &a[0] {
        LispVal::List(v) => {atleast(v, 1)?; Ok(LispVal::List(v[1..].to_vec()))},
        LispVal::DottedList(v, r) => {
            if v.len() == 1 {
                Ok(r.deref().clone())
            } else {
                Ok(DottedList(v[1..].to_vec(), r.clone()))
            }
        },
        _ => Err(Runtime("Expected list".to_string()))
    }
}

fn atleast(v: &[LispVal], i: usize) -> Result<(), LispErr> {
    if v.len() < i {
        return Err(Runtime(format!("Expected at least {}", i).to_string()));
    }
    Ok(())
}

fn eqv(a: &[LispVal], _: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    Ok(Boolean(a[0].eq(&a[1])))
}
fn equal(a: &[LispVal], _: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    Ok(Boolean(a[0].to_string().eq(&a[1].to_string()))) // todo!
}

fn apply(a: &[LispVal], env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let f = &a[0];
    if let List(args) = &a[1] {
        crate::evaluation::call_function(f, args, env)
    } else {
        Err(Runtime("Expected list of arguments".to_string()))
    }
}
pub fn create_eden_env() -> Rc<RefCell<Env>> {
    let env: Rc<RefCell<Env>> = Rc::from(RefCell::new(Env::new()));
    {
        let mut e = env.borrow_mut();
        e.define("+", PrimitiveFunc(|a, _| Ok(Number(a[0].num()? + a[1].num()?)))).unwrap();
        e.define("-", PrimitiveFunc(|a, _| Ok(Number(a[0].num()? - a[1].num()?)))).unwrap();
        e.define("*", PrimitiveFunc(|a, _| Ok(Number(a[0].num()? * a[1].num()?)))).unwrap();
        e.define("/", PrimitiveFunc(|a, _| Ok(Number(a[0].num()? / a[1].num()?)))).unwrap();
        e.define("mod", PrimitiveFunc(|a, _| Ok(Number(a[0].num()? % a[1].num()?)))).unwrap();
        e.define("quotent", PrimitiveFunc(|a, _| Ok(Number(a[0].num()? / a[1].num()?)))).unwrap();
        e.define("remainder", PrimitiveFunc(|a, _| Ok(Number(a[0].num()? + a[1].num()?)))).unwrap();
        e.define("=", PrimitiveFunc(|a, _| Ok(Boolean(a[0].num()? == a[1].num()?)))).unwrap();
        e.define(">", PrimitiveFunc(|a, _| Ok(Boolean(a[0].num()? > a[1].num()?)))).unwrap();
        e.define("<", PrimitiveFunc(|a, _| Ok(Boolean(a[0].num()? < a[1].num()?)))).unwrap();
        e.define("=>", PrimitiveFunc(|a, _| Ok(Boolean(a[0].num()? >= a[1].num()?)))).unwrap();
        e.define("<=", PrimitiveFunc(|a, _| Ok(Boolean(a[0].num()? <= a[1].num()?)))).unwrap();
        e.define("&&", PrimitiveFunc(|a, _| Ok(Boolean(a[0].bool()? && a[1].bool()?)))).unwrap();
        e.define("||", PrimitiveFunc(|a, _| Ok(Boolean(a[0].bool()? || a[1].bool()?)))).unwrap();
        e.define("/=", PrimitiveFunc(|a, _| Ok(Boolean(a[0].bool()? != a[1].bool()?)))).unwrap();
        e.define("string=?", PrimitiveFunc(|a, _| Ok(Boolean(a[0].str()? == (a[1].str()?))))).unwrap();
        e.define("string<?", PrimitiveFunc(|a, _| Ok(Boolean(a[0].str()? < a[1].str()?)))).unwrap();
        e.define("string>?", PrimitiveFunc(|a, _| Ok(Boolean(a[0].str()? > a[1].str()?)))).unwrap();
        e.define("string<=?", PrimitiveFunc(|a, _| Ok(Boolean(a[0].str()? <= a[1].str()?)))).unwrap();
        e.define("string>=?", PrimitiveFunc(|a, _| Ok(Boolean(a[0].str()? >= a[1].str()?)))).unwrap();
        e.define("car", PrimitiveFunc(car)).unwrap();
        e.define("cdr", PrimitiveFunc(cdr)).unwrap();
        e.define("cons", PrimitiveFunc(cons)).unwrap();
        e.define("eqv?", PrimitiveFunc(eqv)).unwrap();
        e.define("equal?", PrimitiveFunc(equal)).unwrap();
        e.define("apply", PrimitiveFunc(apply)).unwrap();
        e.define("load", PrimitiveFunc(load)).unwrap();
    }
    env
}
