use std::collections::HashMap;

//Type Difinitions
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

fn main() {
    println!("Hello, world!");
}
