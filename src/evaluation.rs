use crate::parser::LispVal;
use crate::parser::LispVal::{Atom, Boolean, Func, Number};
use std::collections::HashMap;
use std::ops::Deref;
use std::borrow::BorrowMut;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Env {
    parent: Option<Box<Env>>,
    vars: HashMap<String, LispVal>,
}

impl Env {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn child(&self) -> Self {
        Env { parent: Some(Box::new(self.clone())), vars: HashMap::new() }
    }
}

pub fn eval(v: LispVal, mut env: &mut Box<Env>) -> Result<LispVal, String> {
    match v {
        LispVal::Atom(var) => get_var(var, env),
        LispVal::String(_) => Ok(v),
        LispVal::Number(_) => Ok(v),
        LispVal::Boolean(_) => Ok(v),
        LispVal::Quote(q) => Ok(*q),
        LispVal::List(v) => eval_list(&v, &mut env),
        LispVal::DottedList(_, _) => Ok(v),
        LispVal::Func { .. } => todo!(),
    }
}

fn eval_list(list: &Vec<LispVal>, env: &mut Box<Env>) -> Result<LispVal, String> {
    if let Ok(l) = apply_primitive(list, env) {
        return Ok(l);
    } else {
        let a = [define_func, apply_primitive, define_var, set_var, eval_function];
        match eval_any_of(list, env, &a) {
            Ok(r) => Ok(r),
            Err(e) => Err(e),
        }

        // let a = &list[0];
        // if let Atom(s) = a {
        //     match s.as_str() {
        //         "define" => define_var(list, env),
        //         "set!" => set_var(list, env),
        //         _ => get_var(s.clone(), env),
        //     }
        // } else {
        //     return Err("".to_string());
        // }
    }
}

fn eval_any_of<T>(list: &Vec<LispVal>, env: &mut Box<Env>, f: &[T]) -> Result<LispVal, String>
    where T: Fn(&Vec<LispVal>, &mut Box<Env>) -> Result<LispVal, String> {
    for e in f {
        match e(list, env) {
            Ok(val) => return Ok(val),
            Err(_) => (),
        }
    }
    Err("Function is nor recognised".to_string())
}


fn consume(opt: Option<&LispVal>, e: &str) -> Result<LispVal, String> {
    match opt {
        Some(v) => Ok(v.clone()),
        None => Err(e.to_string()),
    }
}

fn consume_exact(opt: Option<&LispVal>, expected: LispVal) -> Result<LispVal, String> {
    let val = consume(opt, format!("Expected {}", expected).as_str())?;
    if val.eq(&expected) {
        Ok(val)
    } else {
        Err(format!("Expected {}", expected).to_string())
    }
}

fn consume_list(opt: Option<&LispVal>) -> Result<Vec<LispVal>, String> {
    match consume(opt, format!("Expected list").as_str())? {
        LispVal::List(r) => Ok(r),
        _ => Err("Expected list".to_string()),
    }
}


fn nothing_to_consume(opt: Option<&LispVal>) -> Result<(), String> {
    match opt {
        Some(v) => Err(format!("Error unexpected value {}", v)),
        None => Ok(()),
    }
}

fn extract_str_from_atom(r: Result<LispVal, String>) -> Result<String, String> {
    match r {
        Ok(Atom(s)) => Ok(s),
        Err(e) => Err(e),
        Ok(other) => Err(format!("Expected atom but got {}", other)),
    }
}

fn define_var(list: &Vec<LispVal>, env: &mut Box<Env>) -> Result<LispVal, String> {
    let mut iter = list.iter();
    consume_exact(iter.next(), Atom("define".to_string()))?;
    let name = extract_str_from_atom(consume(iter.next(), "Expect variable name"))?;
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(a, env))??;
    nothing_to_consume(iter.next())?;
    env.vars.insert(name, val.clone());
    Ok(val)
}

fn set_var(list: &Vec<LispVal>, env: &mut Box<Env>) -> Result<LispVal, String> {
    let mut iter = list.iter();
    consume_exact(iter.next(), Atom("set!".to_string()))?;
    let name = extract_str_from_atom(consume(iter.next(), "Expect variable name"))?;
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(a, env))??;
    nothing_to_consume(iter.next())?;
    match env.vars.insert(name.clone(), val.clone()) {
        Some(_) => Err(format!("Variable is not defined {}", name)),
        None => Ok(val),
    }
}

fn get_var(name: String, env: &mut Box<Env>) -> Result<LispVal, String> {
    match env.vars.get(&name) {
        Some(v) => Ok(v.clone()),
        None => Err(format!("Variable {} is not defined", name)),
    }
}

fn apply_primitive(list: &Vec<LispVal>, env: &mut Box<Env>) -> Result<LispVal, String> {
    let mut iter = list.iter();
    let operator = consume(iter.next(), "Expect operator")?;
    let left = extract_num_value(consume(iter.next(), "Expect argument")?, env)?;
    let right = extract_num_value(consume(iter.next(), "Expect argument")?, env)?;

    if let Atom(s) = operator {
        match s.as_str() {
            "+" => Ok(Number(left + right)),
            "-" => Ok(Number(left - right)),
            "*" => Ok(Number(left * right)),
            "/" => Ok(Number(left / right)),
            "<" => Ok(Boolean(left < right)),
            ">" => Ok(Boolean(left > right)),
            "=" => Ok(Boolean(left == right)),
            "!=" => Ok(Boolean(left != right)),
            _ => Err(format!("Invalid infix operator: {}", s)),
        }
    } else {
        Err(format!("Operation is not recognised: {}", operator))
    }
}

fn extract_num_value(lv: LispVal, env: &mut Box<Env>) -> Result<i64, String> {
    let left = eval(lv, env)?;
    match left {
        LispVal::Number(n) => Ok(n),
        LispVal::String(s) => Ok(s.parse().unwrap()),
        _ => return Err(format!("Left operand must be an integer {:?}", left)),
    }
}

fn extract_string_value(lv: LispVal, env: &mut Box<Env>) -> Result<String, String> {
    let left = eval(lv, env)?;
    match left {
        LispVal::Number(n) => Ok(n.to_string()),
        LispVal::String(s) => Ok(s),
        _ => return Err(format!("Left operand must be a string {:?}", left)),
    }
}

fn define_func(list: &Vec<LispVal>, env: &mut Box<Env>) -> Result<LispVal, String> {
    let mut iter = list.iter();
    consume_exact(iter.next(), Atom("define".to_string()))?;
    let definition = consume_list(iter.next())?;
    let name = extract_str_from_atom(consume(definition.first(), "Expect function name"))?;
    let params: &Vec<String> = &definition[1..].iter().map(|a| format!("{}", a)).collect();
    let body = consume(iter.next(), "Expect body")?;
    nothing_to_consume(iter.next())?;
    let closure = env.child();
    env.vars.insert(name, Func { args: params.clone(), body: Box::new(body), vararg: None });
    Ok(Atom("".to_string()))
}

fn eval_function(list: &Vec<LispVal>, env: &mut Box<Env>) -> Result<LispVal, String> {
    let mut iter = list.iter();
    let name = extract_str_from_atom(consume(iter.next(), "Expect function name"))?;

    let a = env.vars.get(name.as_str());
    if a.is_none() {
        return Err(format!("Function {} not found", name));
    }
    let Some(Func { args: args, body: body, vararg: vararg }) = a
        else {
            return Err(format!("Incorrect function call"));
        };
    let args = args.clone();
    let body = body.deref().clone();
    let mut closure = Box::new(env.child());
    if list.len() - 1 != args.len() {
        return Err("Incorrect argument list".to_string());
    }

    for (i, arg) in args.iter().enumerate() {
        let arg_val = eval(list[i + 1].clone(), env)?;
        closure.vars.borrow_mut().insert(arg.to_string(), arg_val);
    }
    eval(body, &mut closure)
}