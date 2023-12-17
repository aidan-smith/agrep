mod ast;
mod regex;
mod nfa;

use anyhow::Result;

use crate::nfa::NFA;

fn parse(pattern: &str) -> Result<regex::Regex> {
    let mut parser = ast::Parser::new();
    let ast = parser.parse(pattern)?;
    let regex = regex::Parser::new().parse(&ast);
    Ok(regex)
}

#[allow(dead_code)]
enum Type {
    ERE,
    BRE,
    PCRE,
}

fn main() {
    println!("Hello, world!");

    // let pattern = "a{1,2}(foo|bar)[ac-z]*";
    let pattern = "abc";
    // let ast = ast::Parser::new().parse(pattern);


    let regex = parse(pattern).unwrap();
    // println!("{:#?}", regex);

    let nfa = NFA::from_regex(&regex);
    println!("{:#?}", nfa);
}
