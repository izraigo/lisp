use std::cell::RefCell;
use std::rc::Rc;
use std::fmt::Display;
use crate::env::Env;
use crate::error::LispErr;
use crate::error::LispErr::Runtime;
use crate::lispval::LispVal::Boolean;

#[derive(Clone, Debug, PartialEq)]
pub enum LispVal {
    Atom(String),
    Number(i64),
    LispString(String),
    Boolean(bool),
    List(Vec<LispVal>),
    DottedList(Vec<LispVal>, Box<LispVal>),
    Quote(Box<LispVal>),
    Func {
        args: Vec<String>,
        vararg: Option<String>,
        body: Vec<LispVal>,
        closure: Rc<RefCell<Env>>,
    },
    PrimitiveFunc(fn(&[LispVal], &Rc<RefCell<Env>>) -> Result<LispVal, LispErr>)
}

impl Display for LispVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LispVal::Atom(s) => write!(f, "{}", s),
            LispVal::Number(n) => write!(f, "{}", n),
            LispVal::LispString(s) => write!(f, "\"{}\"", s),
            LispVal::Boolean(b) => write!(f, "{}", b),
            LispVal::List(v) => {
                let a: Vec<String> = v.into_iter().map(|i| i.to_string()).collect();
                write!(f, "({})", a.join(" "))
            }
            LispVal::DottedList(v, v1) => {
                let a: Vec<String> = v.into_iter().map(|i| i.to_string()).collect();
                write!(f, "({} . {})", a.join(" "), v1.to_string())
            }
            LispVal::Quote(q) => write!(f, "quote {}", q),
            LispVal::Func {args, body, .. } => {
                let body: Vec<String> = body.into_iter().map(|i| i.to_string()).collect();
                write!(f, "lambda {} {}", args.join(" "), body.join(" "))
            },
            LispVal::PrimitiveFunc(_) => write!(f, "primitiveFunc"),
        }
    }
}

impl LispVal {
    pub fn num(&self) -> Result<i64, LispErr> {
        match self {
            LispVal::Number(n) => Ok(n.clone()),
            LispVal::LispString(s) => Ok(s.parse().unwrap()),
            _ => return Err(Runtime(format!("Left operand must be an integer {:?}", self))),
        }
    }

    pub fn bool(&self) -> Result<bool, LispErr> {
        match self {
            Boolean(b) => Ok(b.clone()),
            LispVal::Number(n) => Ok(n.clone() != 0),
            LispVal::LispString(s) => Ok(s.parse().unwrap()),
            _ => return Err(Runtime(format!("Left operand must be an integer {:?}", self))),
        }
    }

    pub fn str(&self) -> Result<String, LispErr> {
        match self {
            LispVal::Number(n) => Ok(n.to_string()),
            LispVal::LispString(s) => Ok(s.clone()),
            _ => return Err(Runtime(format!("Left operand must be a string {:?}", self))),
        }
    }
}