#![allow(dead_code)]

use crate::ast::{self, AST};
use anyhow::Result;

type DS = Vec<Regex>;

enum Regex {
    Empty,
    Literal(Box<[char]>),
    Class(DS),
    Assert(ast::AnchorType),
    Repetition(Box<Regex>),
    Concat(Vec<Regex>),
    Alternation(Vec<Regex>),
}

enum RepetitionType {
    Exact(u32),
    Lower(u32),
    Range(u32, u32),
}

struct Parser {
    pos: usize,
}

impl Parser {
    fn new() -> Self {
        Self { pos: 0 }
    }

    fn parse(&mut self, ast: &AST) -> Result<Regex> {
        ParserVM::new(self, ast).parse()
    }
}

struct ParserVM<'a> {
    parser: &'a mut Parser,
    ast: &'a AST,
}

impl<'a> ParserVM<'a> {
    fn new(parser: &'a mut Parser, ast: &'a AST) -> Self {
        Self { parser, ast }
    }

    fn parse(&mut self) -> Result<Regex> {
        Ok(Regex::Empty)
    }
}

