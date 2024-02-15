use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum LispErr {
    Runtime(String),
    Choice(String),
}

impl Display for LispErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LispErr::Runtime(s) => write!(f, "{}", s),
            LispErr::Choice(n) => write!(f, "{}", n),
        }
    }
}