mod parser;

use std::collections::HashMap;
use nom::character::complete::space1;
use nom::multi::{many0, separated_list1};
use nom::{
    IResult,
    sequence::delimited,
    sequence::separated_pair,
    character::complete::char,
    character::complete::digit1,
    character::complete::alpha1,
    character::complete::alphanumeric1,
    character::complete::space0,

    branch::alt,
    bytes::complete::is_not,
    bytes::complete::is_a,
    multi::many1
  };
use std::io::{stdin, Write};
use std::fmt::{Debug, Display};
use crate::LispVal::{Atom, Boolean, Number};

fn main() {
    println!("Lisp in rust!");
    let mut env = Box::from(Env::new());
    loop {
        let mut s = String::new();
        print!("lisp>>> ");
        std::io::stdout().flush();
        stdin().read_line(&mut s).expect("Enter expression");

        match parse_expr(&s) {
            Ok((_, lisp_val)) => {
                match eval(lisp_val, &mut env) {
                    Ok(res) => println!("{}", res),
                    Err(e) => println!("Error: {}", e)
                }
            },
            Err(e) => println!("Error: {}", e)
        }
    }

}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Env {
    parent: Option<Box<Env>>,
    vars: HashMap<String, LispVal>
}

impl Env {
    pub fn new() -> Self {
        Default::default()
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum LispVal {
    Atom(String),
    Number(i64),
    String(String),
    Boolean(bool),
    List(Vec<LispVal>),
    DottedList(Vec<LispVal>, Box<LispVal>),
    Quote(Box<LispVal>),
    Func { args:Vec<String>, vararg: Option<String>, body: Box<LispVal>, closure: Env}
}

impl Display for LispVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LispVal::Atom(s) => write!(f, "{}", s),
            LispVal::Number(n) => write!(f, "{}", n),
            LispVal::String(s) => write!(f, "\"{}\"", s),
            LispVal::Boolean(b) => write!(f, "{}", b),
            LispVal::List(v) =>
            {
                let a: Vec<String> = v.into_iter().map(|i| i.to_string()).collect();
                write!(f, "({})", a.join(" "))
            },
            LispVal::DottedList(v, v1) => {
                let a: Vec<String> = v.into_iter().map(|i| i.to_string()).collect();
                write!(f, "{} . {}", a.join(" "), v1.to_string())
            }
            LispVal::Quote(q) => write!(f, "quote {}", q),
            LispVal:: Func { .. } => write!(f, "Func {}", "")
        }
    }
}



fn parse_string(input: &str) -> IResult<&str, LispVal> {
    delimited(char('"'), is_not("\""), char('"'))(input)
    .map(|( i, o)| (i, LispVal::String(String::from(o))))
    //Ok((input, String::new()))
  }

  #[test]
fn string_parser_test() {
    let output = parse_string("\"hello\"").unwrap();
    assert_eq!(output, ("", LispVal::String("hello".to_owned())) );
}

fn parse_number(input: &str) -> IResult<&str, LispVal> {
    many1(digit1)(input)
    .map(|( i, o) | (i, LispVal::Number(o.join("").parse::<i64>().unwrap())))
}

#[test]
fn number_parser_test() {
    assert!(parse_number("j5").is_err());
    assert!(parse_number("jlsdf").is_err());
    assert_eq!(parse_number("23").unwrap(), ("", LispVal::Number(23)));
}

fn match_symbols(input: String) -> LispVal {
    match input.as_str() {
        "#t" => LispVal::Boolean(true),
        "#f" => LispVal::Boolean(false),
        _ => LispVal::Atom(input),
    }
}

/*fn symbol() -> impl Fn(&str) -> IResult<(&str, &str), &str>{
    is_a("!#$%&|*+-/:<=>?@^_~")
}
*/
fn parse_atom(input: &str) -> IResult<&str, LispVal> {
    let (input, first) = alt((alpha1, symbol))(input)?;
    let (input, rest) = many0(alt((alphanumeric1, symbol)))(input)?;
    Ok((input, match_symbols(format!("{}{}", String::from(first), rest.join("")))))
}

fn symbol(s: &str) -> IResult<&str, &str> {
    is_a("!#$%&|*+-/:<=>?@^_~")(s)
}

#[test]
fn atom_parser_test() {
    assert_eq!(
        parse_atom("$foo").unwrap(),
        ("", LispVal::Atom("$foo".to_owned()))
    );
    assert_eq!(parse_atom("#f").unwrap(), ("", LispVal::Boolean(false)));
}

fn parse_list(input: &str) -> IResult<&str, LispVal> {
    separated_list1(space1, parse_expr)(input)
        .map(|(i, v)| (i, LispVal::List(v)))
}

fn parse_quoted(input: &str) -> IResult<&str, LispVal> {
    let (input, _) = char('\'')(input)?;
    parse_expr(input)
        .map(|(i, l )| (i, LispVal::Quote(Box::new(l))))
}

fn parse_quoted2(input: &str) -> IResult<&str, LispVal> {
    let (input, _) = char('\'')(input)?;
    parse_expr(input)
        .map(|(i, l )| (i, LispVal::List(vec![LispVal::Atom("quote".to_owned()), l])))
}

fn parse_expr(input: &str) -> IResult<&str, LispVal> {
    alt((parse_atom, parse_number, parse_string, parse_quoted, try_parse_list))(input)
}

fn dotted(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    let (input, _) = char('.')(input)?;
    let (input, _) = space0(input)?;
    Ok((input, "."))
}

fn parse_dotted_list(input: &str) -> IResult<&str, LispVal> {
    separated_pair(parse_list, dotted, parse_expr)(input)
    .map(|(i, (head, rest))|
    {
        let list = match head {
            LispVal::List(v) => v,
            _ => panic!("List parser returned a non-list value")
        };
        (i, LispVal::DottedList(list, Box::new(rest)))
    }
)}

fn try_parse_list(input: &str) -> IResult<&str, LispVal> {
    let (input, _) = char('(')(input)?;
    let (input, items) = alt((parse_dotted_list, parse_list))(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, items))
}

#[test]
fn list_parser_test() {
    assert_eq!(
        parse_list("$foo 42 53").unwrap(),
        (
            "",
            LispVal::List(vec!(
                LispVal::Atom("$foo".to_owned()),
                LispVal::Number(42),
                LispVal::Number(53)
            ))
        )
    );
    assert_eq!(
        parse_list("\"foo\" 42 53").unwrap(),
        (
            "",
            LispVal::List(vec!(
                LispVal::String("foo".to_owned()),
                LispVal::Number(42),
                LispVal::Number(53)
            ))
        )
    );
}

#[test]
fn quoted_parser_test() {
    let output = parse_quoted("'52").unwrap();
    assert_eq!(
        output,
        (
            "",
            LispVal::Quote(Box::new(LispVal::Number(52)))
        )
    );
}

#[test]
fn quoted_parser_test2() {
    let output = parse_quoted2("'52").unwrap();
    assert_eq!(
        output,
        (
            "",
            LispVal::List(vec![LispVal::Atom("quote".to_owned()), LispVal::Number(52)])
        )
    );
}


fn eval(v: LispVal, mut env: &mut Box<Env>) -> Result<LispVal, String> {
    match v {
        LispVal::Atom(var) => get_var(var, env),
        LispVal::String(_) => Ok(v),
        LispVal::Number(_) => Ok(v),
        LispVal::Boolean(_) => Ok(v),
        LispVal::Quote(q) => Ok(*q),
        LispVal::List(v) => eval_list(&v, &mut env),
        LispVal::DottedList(_, _) => Ok(v),
        LispVal::Func { .. } => todo!()
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
                _ => get_var(s.clone(), env)
            }
        } else {
            return Err("".to_string())
        }
    }

}

fn consume(opt: Option<&LispVal>, e: &str) -> Result<LispVal, String> {
    match opt {
        Some(v) => Ok(v.clone()),
        None => Err(e.to_string())
    }
}

fn nothing_to_consume(opt: Option<&LispVal>) -> Result<(), String> {
    match opt {
        Some(v) => Err(format!("Error unexpected value {}", v)),
        None => Ok(())
    }
}


fn extract_str_from_atom(r: Result<LispVal, String>) -> Result<String, String> {
    match r {
        Ok(Atom(s)) => Ok(s),
        Err(e) => Err(e),
        Ok(other)=> Err(format!("Expected atom but got {}", other))
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
        None => Ok(val)
    }
}

fn get_var(name: String, env: &mut Box<Env>) -> Result<LispVal, String> {
    match env.vars.get(&name) {
        Some(v) => Ok(v.clone()),
        None => Err(format!("Variable {} is not defined", name))
    }
}



fn apply_primitive(list: &Vec<LispVal>, env: &mut Box<Env> ) -> Result<LispVal, String> {
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




#[test]
fn eval_test() {
    let mut env = Box::from(Env::new());
    let (_, e) = parse_expr("(+ 2 \"3\")").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, &mut env).unwrap();
    println!("Expression input: {}", res);
    assert_eq!(res, Number(5));

    let (_, e) = parse_expr("(- (+ 4 6 3) 3 5 2)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, &mut env).unwrap();
    println!("Expression input: {}", res);
    assert_ne!(res, Number(3));

    let (_, e) = parse_expr("(+ 2 (- 4 1))").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, &mut env).unwrap();
    println!("Expression input: {}", res);
    assert_eq!(res, Number(5));
}
#[test]
fn eval_test2() {
    let mut env = Box::from(Env::new());
    let (_, e) = parse_expr("(define a 2)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, &mut env).unwrap();
    println!("Expression result: {}", res);
    assert_eq!(res, Number(2));

    let (_, e) = parse_expr("(a)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, &mut env).unwrap();
    println!("Expression result: {}", res);
    assert_eq!(res, Number(2));

}
