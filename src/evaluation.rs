use std::cell::RefCell;
use std::rc::Rc;
use crate::env::Env;
use crate::error::LispErr;
use crate::error::LispErr::{WrongExpression, Runtime};
use crate::lispval::LispVal;
use crate::lispval::LispVal::{Atom, Boolean, Func, Number, PrimitiveFunc};

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
        LispVal::PrimitiveFunc(_) => Ok(v),
    }
}

fn eval_list(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let a = [evaluate_if, define_var, define_func, eval_primitive_function_call, set_var, eval_lambda, eval_function_call];
    match eval_any_of(list, env, &a) {
        Ok(r) => Ok(r),
        Err(e) => Err(e),
    }
}

fn eval_any_of<T>(list: &Vec<LispVal>, env: Rc<RefCell<Env>>, f: &[T]) -> Result<LispVal, LispErr>
    where T: Fn(&Vec<LispVal>, Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    for e in f {
        match e(list, env.clone()) {
            Ok(val) => return Ok(val),
            Err(LispErr::Runtime(r)) => return Err(Runtime(r)),
            Err(LispErr::WrongExpression(_)) => (),
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
    to_wrong_expr(consume_exact(iter.next(), Atom("define".to_string())))?;
    let name = match to_wrong_expr(consume(iter.next(), "Expect variable name"))? {
        Atom(s) => s,
        _ => return Err(WrongExpression("Expect variable name".to_string())),
    };
    let val = consume(iter.next(), "Expect variable value").map(|a| eval(a, env.clone()))??;
    nothing_to_consume(iter.next())?;
    env.borrow_mut().define(name, val.clone())
}

fn set_var(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let mut iter = list.iter();
    to_wrong_expr(consume_exact(iter.next(), Atom("set!".to_string())))?;
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

fn unpack_num(lv: &LispVal) -> Result<i64, LispErr> {
    match lv {
        LispVal::Number(n) => Ok(n.clone()),
        LispVal::String(s) => Ok(s.parse().unwrap()),
        _ => return Err(Runtime(format!("Left operand must be an integer {:?}", lv))),
    }
}

fn unpack_bool(lv: &LispVal) -> Result<bool, LispErr> {
    match lv {
        Boolean(b) => Ok(b.clone()),
        LispVal::Number(n) => Ok(n.clone() != 0),
        LispVal::String(s) => Ok(s.parse().unwrap()),
        _ => return Err(Runtime(format!("Left operand must be an integer {:?}", lv))),
    }
}


fn unpack_str(lv: &LispVal) -> Result<String, LispErr> {
    match lv {
        LispVal::Number(n) => Ok(n.to_string()),
        LispVal::String(s) => Ok(s.clone()),
        _ => return Err(Runtime(format!("Left operand must be a string {:?}", lv))),
    }
}

fn define_func(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
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
    env.borrow_mut().define(name, func.clone())
}

fn eval_lambda(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
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

fn eval_primitive_function_call(list: &Vec<LispVal>, env: Rc<RefCell<Env>>) -> Result<LispVal, LispErr> {
    let a = to_wrong_expr(consume(list.first(), "Expect primitive function ").map(|a| eval(a, env.clone()))?);

    let Ok(PrimitiveFunc(f)) = a
        else {
            return Err(WrongExpression(format!("Not a primitive function call")));
        };

    if list.len() < 3 {
        return Err(Runtime("Not enough arguments".to_string()))
    }

    list[2..].iter().try_fold(eval(list[1].clone(), env.clone())?, |acc, xs| f(&acc, &eval(xs.clone(), env.clone())?))
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
    to_wrong_expr(consume_exact(iter.next(), Atom("if".to_string())))?;
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

fn to_wrong_expr(r: Result<LispVal, LispErr>) -> Result<LispVal, LispErr> {
    match r {
        Ok(_) => r,
        Err(Runtime(r)) => Err(WrongExpression(r)),
        _ => r,
    }
}

pub fn create_eden_env() -> Rc<RefCell<Env>> {
    let env = Rc::from(RefCell::new(Env::new()));
    {
        let mut e = env.borrow_mut();
        e.define("+".to_string(), PrimitiveFunc(|a, b| Ok(Number(unpack_num(a)? + unpack_num(b)?))));
        e.define("-".to_string(), PrimitiveFunc(|a, b| Ok(Number(unpack_num(a)? - unpack_num(b)?))));
        e.define("*".to_string(), PrimitiveFunc(|a, b| Ok(Number(unpack_num(a)? * unpack_num(b)?))));
        e.define("/".to_string(), PrimitiveFunc(|a, b| Ok(Number(unpack_num(a)? / unpack_num(b)?))));
        e.define("mod".to_string(), PrimitiveFunc(|a, b| Ok(Number(unpack_num(a)? % unpack_num(b)?))));
        e.define("quotent".to_string(), PrimitiveFunc(|a, b| Ok(Number(unpack_num(a)? / unpack_num(b)?))));
        e.define("remainder".to_string(), PrimitiveFunc(|a, b| Ok(Number(unpack_num(a)? + unpack_num(b)?))));

        e.define("=".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_num(a)? == unpack_num(b)?))));
        e.define(">".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_num(a)? > unpack_num(b)?))));
        e.define("<".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_num(a)? < unpack_num(b)?))));
        e.define("=>".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_num(a)? >= unpack_num(b)?))));
        e.define("<=".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_num(a)? <= unpack_num(b)?))));

        e.define("&&".to_string(), PrimitiveFunc(|a, b| { Ok(Boolean(unpack_bool(a)? && unpack_bool(b)?)) }));
        e.define("||".to_string(), PrimitiveFunc(|a, b| { Ok(Boolean(unpack_bool(a)? || unpack_bool(b)?)) }));
        e.define("/=".to_string(), PrimitiveFunc(|a, b| { Ok(Boolean(unpack_bool(a)? != unpack_bool(b)?)) }));

        e.define("string=?".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_str(a)? == (unpack_str(b)?)))));
        e.define("string<?".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_str(a)? < unpack_str(b)?))));
        e.define("string>?".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_str(a)? > unpack_str(b)?))));
        e.define("string<=?".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_str(a)? <= unpack_str(b)?))));
        e.define("string>=?".to_string(), PrimitiveFunc(|a, b| Ok(Boolean(unpack_str(a)? >= unpack_str(b)?))));
    }
    return env;
}