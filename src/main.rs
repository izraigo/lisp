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
use std::io::stdin;
use std::fmt::{Debug, Display};
use crate::LispVal::{Atom, Boolean, Number};

fn main() {
    println!("Hello, world!");
    let mut s = String::new();
    stdin().read_line(&mut s).expect("Enter exprsssion");
    parse_string(&s).unwrap();
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


fn eval(v: LispVal, env: Box<Env>) -> LispVal {
    match v {
        LispVal::Atom(_) => v,
        LispVal::String(_) => v,
        LispVal::Number(_) => v,
        LispVal::Boolean(_) => v,
        LispVal::Quote(q) => *q,
        LispVal::List(v) => eval_list(&v, &env).unwrap(),
        LispVal::DottedList(_, _) => v,
        LispVal::Func { .. } => todo!()
    }
}

fn eval_list(list: &Vec<LispVal>, env: &Box<Env>) -> Result<LispVal, String> {
    if let Ok(l) = apply_primitive(list, env) {
        return Ok(l);
    } else {
        let a = &list[0];
        if let Atom(s) = a {
            match s.as_str() {
                "" => Ok(Atom("".to_string())),
//todo
                    _ => Err("".to_string())
            };
        };
        return Err("".to_string())
    }

}



fn apply_primitive(list: &Vec<LispVal>, env: &Box<Env> ) -> Result<LispVal, String> {
    let left = extract_num_value(list[1].clone(), env.clone()).unwrap();
    let right = extract_num_value(list[2].clone(), env.clone()).unwrap();

    let operator = list[0].clone();
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

fn extract_num_value(lv: LispVal, env: Box<Env>) -> Result<i64, Result<LispVal, String>> {
    let left = eval(lv, env);
    match left {
        LispVal::Number(n) => Ok(n),
        LispVal::String(s) => Ok(s.parse().unwrap()),
        _ => return Err(Err(format!("Left operand must be an integer {:?}", left))),
    }
}

#[test]
fn eval_test() {
    let env = Box::from(Env::new());
    let (_, e) = parse_expr("(+ 2 \"3\")").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env.clone());
    println!("Expression input: {}", res);
    assert_eq!(res, Number(5));

    let (_, e) = parse_expr("(- (+ 4 6 3) 3 5 2)").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env.clone());
    println!("Expression input: {}", res);
    assert_ne!(res, Number(3));

    let (_, e) = parse_expr("(+ 2 (- 4 1))").unwrap();
    println!("Expression input: {}", e);
    let res = eval(e, env.clone());
    println!("Expression input: {}", res);
    assert_eq!(res, Number(5));
}
