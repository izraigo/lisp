use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum LispVal {
    Atom(String),
    Number(i64),
    String(String),
    Boolean(bool),
    List(Vec<LispVal>),
    DottedList(Vec<LispVal>, Box<LispVal>),
    Quote(Box<LispVal>),
    Func {
        args: Vec<String>,
        vararg: Option<String>,
        body: Box<LispVal>,
    },
}

impl Display for LispVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LispVal::Atom(s) => write!(f, "{}", s),
            LispVal::Number(n) => write!(f, "{}", n),
            LispVal::String(s) => write!(f, "\"{}\"", s),
            LispVal::Boolean(b) => write!(f, "{}", b),
            LispVal::List(v) => {
                let a: Vec<String> = v.into_iter().map(|i| i.to_string()).collect();
                write!(f, "({})", a.join(" "))
            }
            LispVal::DottedList(v, v1) => {
                let a: Vec<String> = v.into_iter().map(|i| i.to_string()).collect();
                write!(f, "{} . {}", a.join(" "), v1.to_string())
            }
            LispVal::Quote(q) => write!(f, "quote {}", q),
            LispVal::Func {args, .. } => write!(f, "Func {}", args.join(" ")),
        }
    }
}
