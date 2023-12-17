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
    let pattern1 = "a{1,2}(foo|bar)[ac-z]*";
    let pattern2 = "foo(baz).*(bar|baz)?";
    let pattern3 = "[a-z]*";

    let regex1 = parse(pattern1).unwrap();
    let regex2 = parse(pattern2).unwrap();
    let regex3 = parse(pattern3).unwrap();

    let nfa1 = NFA::from_regex(&regex1);
    let nfa2 = NFA::from_regex(&regex2);
    let nfa3 = NFA::from_regex(&regex3);
    println!("{:#?}", nfa1);
    println!("{:#?}", nfa2);
    println!("{:#?}", nfa3);
}
