use std::cell::RefCell;
use std::rc::Rc;

use crate::env::Env;
use crate::error::LispErr;
use crate::error::LispErr::{Expected, Runtime, WrongExpression};
use crate::lispval::LispVal;
use crate::lispval::LispVal::{Atom, Boolean, Func, PrimitiveFunc};

pub fn eval(v: &LispVal, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    match v {
        LispVal::Atom(var) => get_var(&var, env),
        LispVal::LispString(_) => Ok(v.clone()),
        LispVal::Number(_) => Ok(v.clone()),
        LispVal::Boolean(_) => Ok(v.clone()),
        LispVal::Quote(q) => Ok(*q.clone()),
        LispVal::List(v) => eval_list(&v, env),
        LispVal::DottedList(_, _) => Ok(v.clone()),
        LispVal::Func { .. } => Ok(v.clone()),
        LispVal::PrimitiveFunc(_) => Ok(v.clone()),
        //LispVal::File(_) => Ok(v.clone()),
    }
}

fn eval_list(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let a = [evaluate_if,
        define_var,
        define_vararg_func,
        define_func,
        set_var,
        eval_lambda,
        eval_function_call];
    match eval_any_of(list, env, &a) {
        Ok(r) => Ok(r),
        Err(e) => Err(e),
    }
}

fn eval_any_of<T>(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>, f: &[T]) -> Result<LispVal, LispErr>
    where T: Fn(&Vec<LispVal>, &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    for e in f {
        match e(list, env) {
            Ok(val) => return Ok(val),
            Err(LispErr::WrongExpression(_)) => (),
            Err(e) => return Err(e),
        }
    }
    Err(Runtime("Invalid expression".to_string()))
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
        Err(Expected(expected))
    }
}


fn consume_list(opt: Option<&LispVal>) -> Result<Vec<LispVal>, LispErr> {
    match consume(opt, format!("Expected list").as_str())? {
        LispVal::List(r) => Ok(r),
        _ => Err(Runtime("Expected list".to_string())),
    }
}

fn consume_dotted_list(opt: Option<&LispVal>) -> Result<(Vec<LispVal>, Box<LispVal>), LispErr> {
    match consume(opt, format!("Expected dotted list").as_str())? {
        LispVal::DottedList(v, r) => Ok((v, r)),
        _ => Err(WrongExpression("Expected list".to_string())),
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

fn define_var(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_wrong_expr(consume_exact(iter.next(), Atom("define".to_string())))?;
    let name = match to_wrong_expr(consume(iter.next(), "Expect variable name"))? {
        Atom(s) => s,
        _ => return Err(WrongExpression("Expect variable name".to_string())),
    };
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(&a, env))??;
    nothing_to_consume(iter.next())?;
    env.borrow_mut().define(&name, val.clone())
}

fn set_var(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_wrong_expr(consume_exact(iter.next(), Atom("set!".to_string())))?;
    let name = extract_str_from_atom(consume(iter.next(), "Expect variable name"))?;
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(&a, env))??;
    nothing_to_consume(iter.next())?;
    env.borrow_mut().set(&name, val.clone())
}

fn get_var(name: &str, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    match env.borrow_mut().get(name) {
        Some(v) => Ok(v.clone()),
        None => Err(Runtime(format!("Variable {} is not defined", name))),
    }
}

fn define_vararg_func(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_wrong_expr(consume_exact(iter.next(), Atom("define".to_string())))?;
    let (definition, vararg) =  consume_dotted_list(iter.next())?;
    let name = extract_str_from_atom(consume(definition.first(), "Expect function name"))?;
    let params: &Vec<String> = &definition[1..].iter().map(|a| format!("{}", a)).collect();

    let mut body: Vec<LispVal> = Vec::new();
    for v in iter {
        body.push(v.clone());
    }
    let vararg = vararg.to_string();
    let func = Func { args: params.clone(), body: body, vararg: Some(vararg), closure: env.clone() };
    env.borrow_mut().define(&name, func.clone())
}

fn define_func(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_wrong_expr(consume_exact(iter.next(), Atom("define".to_string())))?;
    let definition = consume_list(iter.next())?;
    let name = extract_str_from_atom(consume(definition.first(), "Expect function name"))?;
    let params: &Vec<String> = &definition[1..].iter().map(|a| format!("{}", a)).collect();

    let mut body: Vec<LispVal> = Vec::new();
    for v in iter {
        body.push(v.clone());
    }
    let func = Func { args: params.clone(), body: body, vararg: None, closure: env.clone() };
    env.borrow_mut().define(&name, func.clone())
}

fn eval_lambda(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_wrong_expr(consume_exact(iter.next(), Atom("lambda".to_string())))?;
    let definition = consume_list(iter.next())?;
    let params: &Vec<String> = &definition[0..].iter().map(|a| format!("{}", a)).collect();
    let mut body: Vec<LispVal> = Vec::new();
    for v in iter {
        body.push(v.clone());
    }
    Ok(Func { args: params.clone(), body: body, vararg: None, closure: env.clone()})
}
fn eval_function_call(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if list.len() < 1 {
        return Err(Runtime("Expected function".to_string()));
    }
    let list:Result<Vec<LispVal>, LispErr> = list.iter().map(|v|eval(v, &env)).collect();
    let list = list?;
    call_function(&list[0], &list[1..], env)
}

pub fn call_function(f: &LispVal, list: &[LispVal], env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    if list.len() < 1 {
        return Err(Runtime("Expected function".to_string()));
    }

    if let PrimitiveFunc(func) = f {
        return func(&list, env)
    }

    let Func { args, body, vararg, closure } = f
        else {
            return Err(Runtime(format!("Incorrect function call {}", f)));
        };

    if (vararg.is_none() && list.len() != args.len()) || list.len() < args.len() {
        return Err(Runtime("Incorrect argument count".to_string()));
    }

    let closure = Rc::new(RefCell::new(Env::child(closure.clone())));

    for (i, arg) in args.iter().enumerate() {
        _ = closure.borrow_mut().define(arg, list[i].clone());
    }

    if  let Some(vararg_name) = vararg {
        let mut var_arg_value = vec![];
        for i in args.len()..list.len() {
            var_arg_value.push(list[i].clone());
        }
        closure.borrow_mut().define(&vararg_name, LispVal::List(var_arg_value))?;
    }

    let mut result = Err(Runtime("not executed".to_string()));
    for b in body {
        result = eval(&b, &closure);
        if result.is_err() {
            return result;
        }
    }
    result
}

fn evaluate_if(list: &Vec<LispVal>, env: &Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_wrong_expr(consume_exact(iter.next(), Atom("if".to_string())))?;
    let condition = consume(iter.next(), "Expect condition ").map(|a| eval(&a, env))??;
    let left = consume(iter.next(), "Expect expression ")?;
    let right = consume(iter.next(), "Expect expression ")?;
    nothing_to_consume(iter.next())?;
    match condition {
        Boolean(true) => eval(&left, env),
        Boolean(false) => eval(&right, env),
        _ => Err(Runtime(format!("Expected boolean condition {}", condition))),
    }
}

fn to_wrong_expr(r: Result<LispVal, LispErr>) -> Result<LispVal, LispErr> {
    match r {
        Ok(_) => r,
        Err(e) => Err(WrongExpression(e.to_string())),
    }
}