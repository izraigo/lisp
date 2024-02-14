use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not};
use nom::character::complete::{alpha1, alphanumeric1, char, digit1, space0, space1};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;
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

fn parse_string(input: &str) -> IResult<&str, LispVal> {
    delimited(char('"'), is_not("\""), char('"'))(input)
        .map(|(i, o)| (i, LispVal::String(String::from(o))))
    //Ok((input, String::new()))
}

#[test]
fn string_parser_test() {
    let output = parse_string("\"hello\"").unwrap();
    assert_eq!(output, ("", LispVal::String("hello".to_owned())));
}

fn parse_number(input: &str) -> IResult<&str, LispVal> {
    many1(digit1)(input).map(|(i, o)| (i, LispVal::Number(o.join("").parse::<i64>().unwrap())))
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
    Ok((
        input,
        match_symbols(format!("{}{}", String::from(first), rest.join(""))),
    ))
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
    separated_list1(space1, parse_expr)(input).map(|(i, v)| (i, LispVal::List(v)))
}

fn parse_quoted(input: &str) -> IResult<&str, LispVal> {
    let (input, _) = char('\'')(input)?;
    parse_expr(input).map(|(i, l)| (i, LispVal::Quote(Box::new(l))))
}

fn parse_quoted2(input: &str) -> IResult<&str, LispVal> {
    let (input, _) = char('\'')(input)?;
    parse_expr(input).map(|(i, l)| (i, LispVal::List(vec![LispVal::Atom("quote".to_owned()), l])))
}

pub fn parse_expr(input: &str) -> IResult<&str, LispVal> {
    alt((
        parse_atom,
        parse_number,
        parse_string,
        parse_quoted,
        try_parse_list,
    ))(input)
}

fn dotted(input: &str) -> IResult<&str, &str> {
    let (input, _) = space0(input)?;
    let (input, _) = char('.')(input)?;
    let (input, _) = space0(input)?;
    Ok((input, "."))
}

fn parse_dotted_list(input: &str) -> IResult<&str, LispVal> {
    separated_pair(parse_list, dotted, parse_expr)(input).map(|(i, (head, rest))| {
        let list = match head {
            LispVal::List(v) => v,
            _ => panic!("List parser returned a non-list value"),
        };
        (i, LispVal::DottedList(list, Box::new(rest)))
    })
}

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
    assert_eq!(output, ("", LispVal::Quote(Box::new(LispVal::Number(52)))));
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
