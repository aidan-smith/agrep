#![allow(dead_code)]

use crate::ast::{self, AST};

type DS = Vec<Regex>;

#[derive(Debug)]
pub enum Regex {
    Empty,
    Literal(Box<[char]>),
    Class {
        negated: bool,
        items: Vec<ast::ClassItem>,
    },
    Assert(ast::AnchorType),
    Repetition(RepetitionType, Box<Regex>),
    Concat(Vec<Regex>),
    Alternation(Vec<Regex>),
}

#[derive(Debug, Clone)]
pub enum RepetitionType {
    Exact(u32),
    Lower(u32),
    Range(u32, u32),
}

pub struct Parser {
    pos: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn parse(&mut self, ast: &AST) -> Regex {
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

    fn parse_node(&mut self, ast: &AST) -> Regex {
        match ast {
            AST::Empty => Regex::Empty,
            AST::Wildcard => Regex::Class {
                negated: false,
                items: vec![ast::ClassItem::Range {
                    start: 0.into(),
                    end: char::MAX,
                }],
            },
            AST::Literal(literal) => Regex::Literal(vec![*literal].into_boxed_slice()),
            AST::Class { negated, items } => Regex::Class {
                negated: negated.clone(),
                items: items.clone(),
            },
            AST::Anchor(anchor_type) => Regex::Assert(anchor_type.clone()),
            AST::Repetition(repetition_type, ast) => {
                let rep = match repetition_type {
                    ast::RepetitionType::ZeroOrOne => RepetitionType::Range(0, 1),
                    ast::RepetitionType::ZeroOrMore => RepetitionType::Lower(0),
                    ast::RepetitionType::OneOrMore => RepetitionType::Lower(1),
                    ast::RepetitionType::Exact(n) => RepetitionType::Exact(*n),
                    ast::RepetitionType::Lower(n) => RepetitionType::Lower(*n),
                    ast::RepetitionType::Range(n, m) => RepetitionType::Range(*n, *m),
                };
                Regex::Repetition(rep, Box::new(self.parse_node(ast)))
            }
            AST::Concat(ast) => Regex::Concat(ast.iter().map(|ast| self.parse_node(ast)).collect()),
            AST::Alternation(ast) => {
                Regex::Alternation(ast.iter().map(|ast| self.parse_node(ast)).collect())
            }
            AST::Group(ast) => self.parse_node(ast),
        }
    }

    fn parse(&mut self) -> Regex {
        self.parse_node(self.ast)
    }
}
