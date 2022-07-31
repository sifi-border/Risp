use core::fmt;
use std::{
    collections::HashMap,
    io::{self, Write},
    num::ParseFloatError,
};

//Type Definitions
#[derive(Clone)]
enum RispExp {
    Bool(bool),
    Symbol(String),
    Number(f64),
    List(Vec<RispExp>),
    Func(fn(&[RispExp]) -> Result<RispExp, RispErr>),
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
    match token.as_ref() {
        "true" => RispExp::Bool(true),
        "false" => RispExp::Bool(false),
        _ => {
            let potential_float: Result<f64, ParseFloatError> = token.parse();
            match potential_float {
                Ok(v) => RispExp::Number(v),
                Err(_) => RispExp::Symbol(token.to_string()),
            }
        }
    }
}

fn parse_list_of_floats(args: &[RispExp]) -> Result<Vec<f64>, RispErr> {
    args.iter().map(|x| parse_single_float(x)).collect()
}

fn parse_single_float(exp: &RispExp) -> Result<f64, RispErr> {
    match exp {
        RispExp::Number(num) => Ok(*num),
        _ => Err(RispErr::Reason("expected a number".to_string())),
    }
}

// Environment

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {{
        |args: &[RispExp]| -> Result<RispExp, RispErr> {
            let floats = parse_list_of_floats(args)?;
            let first = floats
                .first()
                .ok_or(RispErr::Reason("expected at least one number".to_string()))?;
            let rest = &floats[1..];
            fn f(prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
                    None => true,
                }
            };
            Ok(RispExp::Bool(f(first, rest)))
        }
    }};
}

fn default_env() -> RispEnv {
    let mut data: HashMap<String, RispExp> = HashMap::new();
    // add "+" func
    data.insert(
        "+".to_string(),
        RispExp::Func(|args: &[RispExp]| -> Result<RispExp, RispErr> {
            let sum = parse_list_of_floats(args)?
                .iter()
                .fold(0.0, |sum, a| sum + a);
            Ok(RispExp::Number(sum))
        }),
    );
    // add "-" func
    data.insert(
        "-".to_string(),
        RispExp::Func(|args: &[RispExp]| -> Result<RispExp, RispErr> {
            let floats = parse_list_of_floats(args)?;
            let first = *floats
                .first()
                .ok_or(RispErr::Reason("expexted at least one number".to_string()))?;
            let sum_of_rest = floats[1..].iter().fold(0.0, |sum, a| sum + a);
            Ok(RispExp::Number(first - sum_of_rest))
        }),
    );

    //Bool functions
    data.insert(
        "=".to_string(),
        RispExp::Func(ensure_tonicity!(|a, b| a == b)),
    );
    data.insert(
        ">".to_string(),
        RispExp::Func(ensure_tonicity!(|a, b| a > b)),
    );
    data.insert(
        "<".to_string(),
        RispExp::Func(ensure_tonicity!(|a, b| a < b)),
    );
    data.insert(
        ">=".to_string(),
        RispExp::Func(ensure_tonicity!(|a, b| a >= b)),
    );
    data.insert(
        "<=".to_string(),
        RispExp::Func(ensure_tonicity!(|a, b| a <= b)),
    );

    RispEnv { data }
}

//Evaluation
fn eval(exp: &RispExp, env: &mut RispEnv) -> Result<RispExp, RispErr> {
    match exp {
        RispExp::Bool(_a) => Ok(exp.clone()),
        RispExp::Symbol(k) => env
            .data
            .get(k)
            .ok_or(RispErr::Reason(format!("unexpected symbol k='{}'", k)))
            .map(|x| x.clone()),
        RispExp::Number(_) => Ok(exp.clone()),
        RispExp::List(list) => {
            let first_form = list
                .first()
                .ok_or(RispErr::Reason("expeceted a non-empty list".to_string()))?;
            let arg_forms = &list[1..];
            let first_eval = eval(first_form, env)?;
            match first_eval {
                RispExp::Func(f) => {
                    let args_eval = arg_forms
                        .iter()
                        .map(|x| eval(x, env))
                        .collect::<Result<Vec<RispExp>, RispErr>>();
                    f(&args_eval?)
                }
                _ => Err(RispErr::Reason("first form must be a function".to_string())),
            }
        }
        RispExp::Func(_) => Err(RispErr::Reason("unexpected form".to_string())),
    }
}

// For Repl (read-eval-print-loop)
impl fmt::Display for RispExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            RispExp::Bool(a) => a.to_string(),
            RispExp::Symbol(s) => s.clone(),
            RispExp::Number(n) => n.to_string(),
            RispExp::List(list) => {
                let xs = list.iter().map(|x| x.to_string()).collect::<Vec<String>>();
                format!("({})", xs.join(","))
            }
            RispExp::Func(_) => "Function {}".to_string(), //ここ呼ばれることあるんか？
        };

        write!(f, "{}", str)
    }
}

fn parse_and_eval(expr: String, env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (parsed_exp, _) = parse(&tokenize(expr))?;
    let evaled_exp = eval(&parsed_exp, env)?;

    Ok(evaled_exp)
}

fn read_input_expr() -> String {
    let mut expr = String::new();
    io::stdin()
        .read_line(&mut expr)
        .expect("Failed to read line");
    expr
}

fn main() {
    let env = &mut default_env();
    loop {
        print!("risp > ");
        io::stdout().flush().unwrap(); // flush to show prompt
        let input_expr = read_input_expr();
        if input_expr == "quit\n".to_string() {
            break;
        }
        match parse_and_eval(input_expr, env) {
            Ok(res) => println!("// 🔥 => {}", res),
            Err(e) => match e {
                RispErr::Reason(msg) => println!("// 🙀 => {}", msg),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{default_env, parse_and_eval, tokenize, RispExp};

    #[test]
    fn tokenize_check() {
        assert_eq!(
            tokenize("(+ 10 5)".to_string()),
            vec!["(", "+", "10", "5", ")"]
        )
    }

    #[test]
    fn eval_check() {
        let env = &mut default_env();
        assert_eq!(
            format!(
                "{}",
                parse_and_eval("(+ 10 5 (- 10 3 3))".to_string(), env).unwrap()
            ),
            "19"
        );
        assert_eq!(
            format!(
                "{}",
                parse_and_eval("(> 6 4 3 1)".to_string(), env).unwrap()
            ),
            "true"
        );
    }
}
