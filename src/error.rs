use std::fmt::Display;
use crate::lispval::LispVal;

#[derive(Clone, Debug, PartialEq)]
pub enum LispErr {
    Runtime(String),
    WrongExpression(String),
    Expected(LispVal),
}

impl Display for LispErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LispErr::Runtime(s) => write!(f, "{}", s),
            LispErr::WrongExpression(n) => write!(f, "{}", n),
            LispErr::Expected(v) => write!(f, "Expected {}", v)
        }
    }
}