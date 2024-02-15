use std::cell::RefCell;
use std::rc::Rc;
use crate::env::Env;
use crate::lispErr::LispErr;
use crate::lispErr::LispErr::{Choice, Runtime};
use crate::lispval::LispVal;
use crate::lispval::LispVal::{Atom, Boolean, Func, Number};

pub fn eval(v: LispVal, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    match v {
        LispVal::Atom(var) => get_var(var, env),
        LispVal::String(_) => Ok(v),
        LispVal::Number(_) => Ok(v),
        LispVal::Boolean(_) => Ok(v),
        LispVal::Quote(q) => Ok(*q),
        LispVal::List(v) => eval_list(&v, env),
        LispVal::DottedList(_, _) => Ok(v),
        LispVal::Func { .. } => Ok(v),
    }
}

fn eval_list(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if let Ok(l) = apply_primitive(list, env.clone()) {
        return Ok(l);
    } else {
        let a = [evaluate_if, define_var, define_func, apply_primitive, set_var, eval_lambda, eval_function_call];
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

fn eval_any_of<T>(list: &Vec<LispVal>, env: Rc<RefCell<Env>>, f: &[T]) -> Result<LispVal, LispErr>
    where T: Fn(&Vec<LispVal>, Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    for e in f {
        match e(list, env.clone()) {
            Ok(val) => return Ok(val),
            Err(LispErr::Runtime(r)) => return Err(Runtime(r)),
            Err(LispErr::Choice(_)) => (),
        }
    }
    Err(Runtime("Invalid expression".to_string()))
    //Err(format!("Function is not recognised {}", LispVal::List(list.clone())))
}


fn consume(opt: Option<&LispVal>, e: &str) -> Result<LispVal, LispErr> {
    match opt {
        Some(v) => Ok(v.clone()),
        None => Err(Runtime(e.to_string())),
    }
}

fn consume_exact(opt: Option<&LispVal>, expected: LispVal) -> Result<LispVal, LispErr> {
    let val = consume(opt, format!("Expected {}", expected).as_str())?;
    if val.eq(&expected) {
        Ok(val)
    } else {
        Err(Runtime(format!("Expected {}", expected).to_string()))
    }
}

fn consume_atom(opt: Option<&LispVal>) -> Result<LispVal, LispErr> {
    let val = consume(opt, "Expected atom")?;
    match val {
        Atom(_) => Ok(val),
        _ => Err(Runtime("Expected atom".to_string()))
    }
}


fn consume_list(opt: Option<&LispVal>) -> Result<Vec<LispVal>, LispErr> {
    match consume(opt, format!("Expected list").as_str())? {
        LispVal::List(r) => Ok(r),
        _ => Err(Runtime("Expected list".to_string())),
    }
}

fn nothing_to_consume(opt: Option<&LispVal>) -> Result<(), LispErr> {
    match opt {
        Some(v) => Err(Runtime(format!("Error unexpected value {}", v))),
        None => Ok(()),
    }
}

fn extract_str_from_atom(r: Result<LispVal, LispErr>) -> Result<String, LispErr> {
    match r {
        Ok(Atom(s)) => Ok(s),
        Err(e) => Err(e),
        Ok(other) => Err(Runtime(format!("Expected atom but got {}", other))),
    }
}

fn define_var(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_choice(consume_exact(iter.next(), Atom("define".to_string())))?;
    let name = match to_choice(consume(iter.next(), "Expect variable name"))? {
        Atom(s) => s,
        _ => return Err(Choice("Expect variable name".to_string())),
    };
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(a, env.clone()))??;
    nothing_to_consume(iter.next())?;
    env.borrow_mut().define(name, val.clone())
}

fn set_var(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_choice(consume_exact(iter.next(), Atom("set!".to_string())))?;
    let name = extract_str_from_atom(consume(iter.next(), "Expect variable name"))?;
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(a, env.clone()))??;
    nothing_to_consume(iter.next())?;
    env.borrow_mut().set(name.clone(), val.clone())
}

fn get_var(name: String, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    match env.borrow_mut().get(&name) {
        Some(v) => Ok(v.clone()),
        None => Err(Runtime(format!("Variable {} is not defined", name))),
    }
}

fn apply_primitive(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    let operator = to_choice(consume(iter.next(), "Expect operator"))?;

    if let Atom(s) = operator {
        let fun = match s.as_str() {
            "+" => {|a, b| {Number(a + b)}},
            "-" => {|a, b| {Number(a - b)}},
            "*" => {|a, b| {Number(a * b)}},
            "/" => {|a, b| {Number(a / b)}},
            "<" => {|a, b| {Boolean(a < b)}},
            ">" => {|a, b| {Boolean(a > b)}},
            "=" => {|a, b| {Boolean(a == b)}},
            "!=" => {|a, b| {Boolean(a != b)}},
            _ => return Err(Choice(format!("Invalid infix operator: {}", s))),
        };
        let left = extract_num_value(consume(iter.next(), "Expect argument")?, env.clone())?;
        let right = extract_num_value(consume(iter.next(), "Expect argument")?, env)?;
        Ok(fun(left, right))
    } else {
        Err(Runtime(format!("Operation is not recognised: {}", operator)))
    }
}

fn extract_num_value(lv: LispVal, env: Rc<RefCell<Env>>) -> Result<i64, LispErr> {
    let left = eval(lv, env)?;
    match left {
        LispVal::Number(n) => Ok(n),
        LispVal::String(s) => Ok(s.parse().unwrap()),
        _ => return Err(Runtime(format!("Left operand must be an integer {:?}", left))),
    }
}

fn extract_string_value(lv: LispVal, env: Rc<RefCell<Env>>) -> Result<String, LispErr> {
    let left = eval(lv, env)?;
    match left {
        LispVal::Number(n) => Ok(n.to_string()),
        LispVal::String(s) => Ok(s),
        _ => return Err(Runtime(format!("Left operand must be a string {:?}", left))),
    }
}

fn define_func(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_choice(consume_exact(iter.next(), Atom("define".to_string())))?;
    let definition = consume_list(iter.next())?;
    let name = extract_str_from_atom(consume(definition.first(), "Expect function name"))?;
    let params: &Vec<String> = &definition[1..].iter().map(|a| format!("{}", a)).collect();

    let mut body: Vec<LispVal> = Vec::new();
    for v in iter {
        body.push(v.clone());
    }
    let func = Func { args: params.clone(), body: body, vararg: None, closure: env.clone() };
    env.borrow_mut().define(name, func.clone())
}

fn eval_lambda(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_choice(consume_exact(iter.next(), Atom("lambda".to_string())))?;
    let definition = consume_list(iter.next())?;
    let params: &Vec<String> = &definition[0..].iter().map(|a| format!("{}", a)).collect();
    let mut body: Vec<LispVal> = Vec::new();
    for v in iter {
        body.push(v.clone());
    }
    Ok(Func { args: params.clone(), body: body, vararg: None, closure: env.clone()})
}

fn eval_function_call(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let a = consume(list.first(), "Expect condition ").map(|a| eval(a, env.clone()))?;

    let Ok(Func { args, body, vararg: _vararg, closure }) = a
        else {
            return Err(Runtime(format!("Incorrect function call")));
        };
    let args = args.clone();

    if list.len() - 1 != args.len() {
        return Err(Runtime("Incorrect argument list".to_string()));
    }

    let closure = Rc::new(RefCell::new(Env::child(closure)));
    for (i, arg) in args.iter().enumerate() {
        let arg_val = eval(list[i + 1].clone(), env.clone())?;
        _ = closure.borrow_mut().define(arg.to_string(), arg_val);
    }

    let mut result = Err(Runtime("not executed".to_string()));
    for b in body {
        result = eval(b.clone(), closure.clone());
        if result.is_err() {
            return result;
        }
    }
    result
}

fn evaluate_if(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_choice(consume_exact(iter.next(), Atom("if".to_string())))?;
    let condition = consume(iter.next(), "Expect condition ").map(|a| eval(a, env.clone()))??;
    let left = consume(iter.next(), "Expect expression ")?;
    let right = consume(iter.next(), "Expect expression ")?;
    nothing_to_consume(iter.next())?;
    match condition {
        Boolean(true) => eval(left, env),
        Boolean(false) => eval(right, env),
        _ => Err(Runtime(format!("Expected boolean condition {}", condition))),
    }
}

fn to_choice(r: Result<LispVal, LispErr>) -> Result<LispVal, LispErr> {
    match r {
        Ok(_) => r,
        Err(Runtime(r)) => Err(Choice(r)),
        _ => r,
    }
}