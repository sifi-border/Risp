use std::{collections::HashMap, num::ParseFloatError};

//Type Definitions
#[derive(Clone)]
enum RispExp {
    Symbol(String),
    Number(f64),
    List(Vec<RispExp>),
}
#[derive(Debug)]
enum RispErr {
    Reason(String),
}
#[derive(Clone)]
struct RispEnv {
    data: HashMap<String, RispExp>,
}

//Parsing
//tokenize("(+ 10 5)") //=> ["(", "+", "10", "5", ")"]
fn tokenize(expr: String) -> Vec<String> {
    expr.replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

fn parse(tokens: &[String]) -> Result<(RispExp, &[String]), RispErr> {
    let (token, rest) = tokens
        .split_first()
        .ok_or(RispErr::Reason("could not get token".to_string()))?;
    match &token[..] {
        "(" => read_seq(rest),
        ")" => Err(RispErr::Reason("unexpexted ')'".to_string())),
        _ => Ok((parse_atom(token), rest)),
    }
}

// reading and parsing the tokens which follows current opening parenthesis, 
// until we hit a closing parenthesis:
fn read_seq(tokens: &[String]) -> Result<(RispExp, &[String]), RispErr> {
    let mut res: Vec<RispExp> = vec![];
    let mut xs = tokens;
    loop {
        let (next_token, rest) = xs
            .split_first()
            .ok_or(RispErr::Reason("could not find closing ')'".to_string()))?;
        if next_token == ")" {
            return Ok((RispExp::List(res), rest));
        }
        let (exp, new_xs) = parse(&xs)?;
        res.push(exp);
        xs = new_xs;
    }
}

fn parse_atom(token: &str) -> RispExp {
    let potential_float: Result<f64, ParseFloatError> = token.parse();
    match potential_float {
        Ok(v) => RispExp::Number(v),
        Err(_) => RispExp::Symbol(token.to_string()),
    }
}
fn main() {
    println!("Hello, world!");
}
