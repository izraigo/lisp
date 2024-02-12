use crate::parser::LispVal;
use crate::parser::LispVal::{Atom, Boolean, Number};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Env {
    parent: Option<Box<Env>>,
    vars: HashMap<String, LispVal>,
}

impl Env {
    pub fn new() -> Self {
        Default::default()
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
        let a = &list[0];
        if let Atom(s) = a {
            match s.as_str() {
                "define" => define_var(list, env),
                "set!" => set_var(list, env),
                _ => get_var(s.clone(), env),
            }
        } else {
            return Err("".to_string());
        }
    }
}

fn consume(opt: Option<&LispVal>, e: &str) -> Result<LispVal, String> {
    match opt {
        Some(v) => Ok(v.clone()),
        None => Err(e.to_string()),
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
    consume(iter.next(), "Expect define")?;
    let name = extract_str_from_atom(consume(iter.next(), "Expect variable name"))?;
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(a, env))??;
    nothing_to_consume(iter.next())?;
    env.vars.insert(name, val.clone());
    Ok(val)
}

fn set_var(list: &Vec<LispVal>, env: &mut Box<Env>) -> Result<LispVal, String> {
    let mut iter = list.iter();
    consume(iter.next(), "Expect set!")?;
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
