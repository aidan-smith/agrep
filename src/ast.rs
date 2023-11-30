#![allow(dead_code)]
use anyhow::{bail, Context, Result};
// use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AST {
    Empty,
    Literal(char),
    Wildcard,
    Anchor(AnchorType),
    Class(bool, Vec<ClassItem>), // negated: bool
    Group(Box<AST>),
    Repetition(RepetitionType, Box<AST>),
    Concat(Vec<AST>),
    Alternation(Vec<AST>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnchorType {
    LineStart, // '^'
    LineEnd,   // '$'
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassItem {
    Ordinary(char),        // 'a'
    Range(char, char),     // 'A-z'
    Collating,             // '[.abc.]'
    Equivalence(char),     // '[=a=]'
    Character(NamedClass), // '[:alpha:]'
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamedClass {
    Alnum,
    Alpha,
    Blank,
    Cntrl,
    Digit,
    Graph,
    Lower,
    Print,
    Punct,
    Space,
    Upper,
    XDigit,
}

impl NamedClass {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "alnum" => Some(Self::Alnum),
            "alpha" => Some(Self::Alpha),
            "blank" => Some(Self::Blank),
            "cntrl" => Some(Self::Cntrl),
            "digit" => Some(Self::Digit),
            "graph" => Some(Self::Graph),
            "lower" => Some(Self::Lower),
            "print" => Some(Self::Print),
            "punct" => Some(Self::Punct),
            "space" => Some(Self::Space),
            "upper" => Some(Self::Upper),
            "xdigit" => Some(Self::XDigit),
            _ => None,
        }
    }
}

// struct Repetition {
//     kind: RepetitionType,
//     ast: AST,

// }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepetitionType {
    ZeroOrOne,       // '?'
    ZeroOrMore,      // '*'
    OneOrMore,       // '+'
    Exact(u32),      // '{n}'
    Lower(u32),      // '{n,}'
    Range(u32, u32), // '{m,n}'
}

pub struct Parser {
    offset: usize,
    group_stack: Vec<Vec<AST>>,
    class_stack: Vec<Vec<AST>>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            offset: 0,
            group_stack: Vec::new(),
            class_stack: Vec::new(),
        }
    }

    pub fn parse(&mut self, pattern: &str) -> Result<AST> {
        ParserVM::new(self, pattern).parse()
    }

    fn reset(&mut self) {
        self.offset = 0;
    }
}

// #[derive(Error, Debug)]
// enum Error {
//     #[error("Unclosed repetition range")]
//     UnclosedRepetitionRange,
// }
//
// type Result<T> = core::result::Result<T, Error>;

struct ParserVM<'a> {
    parser: &'a mut Parser,
    pattern: &'a str,
}

impl<'a> ParserVM<'a> {
    fn new(parser: &'a mut Parser, pattern: &'a str) -> Self {
        Self { parser, pattern }
    }

    fn char(&self) -> char {
        self.pattern[self.parser.offset..].chars().next().unwrap()
    }

    fn is_eof(&self) -> bool {
        self.parser.offset >= self.pattern.len()
    }

    fn peek(&self) -> Option<char> {
        self.pattern[self.parser.offset + self.char().len_utf8()..]
            .chars()
            .next()
    }

    fn next(&mut self) -> bool {
        if self.is_eof() {
            return false;
        }
        self.parser.offset += self.char().len_utf8();
        self.pattern[self.parser.offset..].chars().next().is_some()
    }

    #[allow(dead_code)]
    fn next_ok(&mut self) -> Result<()> {
        if !self.next() {
            bail!(
                "Unexpected EOF after '{}' at offset {}",
                self.char(),
                self.parser.offset
            );
        }
        Ok(())
    }

    fn strip(&mut self) -> bool {
        while !self.is_eof() && self.char().is_whitespace() {
            self.next();
        }
        !self.is_eof()
    }

    fn next_strip(&mut self) -> bool {
        self.next();
        self.strip()
    }

    // TODO: check for overflow -> currently panics
    // TODO: Should error if no digits are found!
    fn parse_int(&mut self) -> Result<u32> {
        let mut num = 0;
        self.strip();
        while !self.is_eof() && self.char().is_digit(10) {
            num = num * 10 + self.char().to_digit(10).unwrap() as u32;
            self.next_strip();
        }
        // TODO: do some validation?
        Ok(num)
    }

    fn start_group(&mut self) {
        todo!()
    }

    fn end_group(&mut self) {
        todo!()
    }

    fn parse_alternate(&mut self) {
        todo!()
    }

    fn parse_enclosed_class(&mut self) -> Result<ClassItem> {
        todo!()
    }

    fn parse_class(&mut self) -> Result<AST> {
        assert!(self.char() == '[');
        if !self.next() {
            bail!("Invalid class: unexpected eof after '['");
        }

        let mut list = vec![];
        let negated = if self.char() == '^' {
            if !self.next() {
                bail!("Invalid class: unexpected eof after '[^'");
            }
            true
        } else {
            false
        };

        // Note: ']' and '-' are ordinary characters if they are the first (or after negation).
        if self.char() == ']' || self.char() == '-' {
            list.push(ClassItem::Ordinary(self.char()));
            if !self.next() {
                bail!("Invalid class: unexpected eof after '{}'", self.char());
            }
        }

        // Collating element can be part of range, but collating symbol cannot.
        // May consider representing differently.
        while self.char() != ']' {
            match self.char() {
                '[' => {
                    let item = self.parse_enclosed_class()?;
                    list.push(item);
                }
                _ => {
                    if let Some('-') = self.peek() {
                        let start = self.char();
                        if !self.next() || !self.next() {
                            bail!("Invalid class: unexpected eof after '{}-'", start);
                        }
                        let end = self.char();
                        if start >= end {
                            bail!(
                                "Invalid class: start '{}' greater than or equal to end '{}'",
                                start,
                                end
                            );
                        }
                        list.push(ClassItem::Range(start, end));
                    } else {
                        list.push(ClassItem::Ordinary(self.char()));
                    }
                    if !self.next() {
                        bail!("Invalid class: unexpected eof");
                    }
                }
            }
        }
        self.next();
        Ok(AST::Class(negated, list))
    }

    fn parse_repetition(&mut self, mut stack: Vec<AST>, rep: RepetitionType) -> Result<Vec<AST>> {
        assert!(
            self.char() == '?' || self.char() == '*' || self.char() == '+' || self.char() == '}'
        );
        self.next();
        let ast = stack
            .pop()
            .context("Invalid repetition: no AST on concat stack")?;
        if let AST::Empty = ast {
            bail!("Invalid repetition: empty AST on concat stack");
        }

        stack.push(AST::Repetition(rep, Box::new(ast)));
        Ok(stack)
    }

    fn parse_repetition_range(&mut self) -> Result<RepetitionType> {
        assert!(self.char() == '{');
        if !self.next_strip() {
            bail!("Invalid repetition range: unexpected eof after '{{'");
        }

        let first = self
            .parse_int()
            .context("Invalid repetition range: no count found")?;
        if self.is_eof() {
            bail!(
                "Invalid repetition range: unexpected eof after '{{{}'",
                first
            );
        }
        Ok(match self.char() {
            ',' => {
                if !self.next_strip() {
                    bail!(
                        "Invalid repetition range: unexpected eof after '{{{},'",
                        first
                    );
                }
                if self.char() == '}' {
                    RepetitionType::Lower(first)
                } else {
                    let second = self.parse_int()?;
                    if first > second {
                        bail!(
                            "Invalid repetition range: first count '{}' is greater than second count '{}'",
                            first,
                            second
                        );
                    }
                    if self.is_eof() || self.char() != '}' {
                        bail!(
                            "Invalid repetition range: unexpected eof/char after '{{{},{}'",
                            first,
                            second
                        );
                    }
                    RepetitionType::Range(first, second)
                }
            }
            '}' => RepetitionType::Exact(first),
            _ => bail!(
                "Invalid repetition range: expected ',' or '}}' but found '{}'",
                self.char()
            ),
        })
    }

    fn parse_primitive(&mut self) -> Result<AST> {
        let prim = match self.char() {
            '\\' => todo!(), // TODO: escape sequences
            '.' => AST::Wildcard,
            '^' => AST::Anchor(AnchorType::LineStart),
            '$' => AST::Anchor(AnchorType::LineEnd),
            _ => AST::Literal(self.char()),
        };
        self.next();
        Ok(prim)
    }

    fn parse(&mut self) -> Result<AST> {
        self.parser.reset();
        let mut stack = vec![];
        while !self.is_eof() {
            match self.char() {
                '(' => self.start_group(),
                ')' => self.end_group(),
                '|' => self.parse_alternate(),
                '[' => stack.push(self.parse_class()?),
                '?' => stack = self.parse_repetition(stack, RepetitionType::ZeroOrOne)?,
                '*' => stack = self.parse_repetition(stack, RepetitionType::ZeroOrMore)?,
                '+' => stack = self.parse_repetition(stack, RepetitionType::OneOrMore)?,
                '{' => {
                    let rep = self.parse_repetition_range()?;
                    stack = self.parse_repetition(stack, rep)?;
                }
                _ => stack.push(self.parse_primitive()?),
            }
        }

        match stack.len() {
            0 => Ok(AST::Empty),
            1 => Ok(stack.pop().unwrap()),
            _ => Ok(AST::Concat(stack)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("a")?;
        assert_eq!(ast, AST::Literal('a'));
        Ok(())
    }

    #[test]
    fn test_literal_concat() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("ab")?;
        assert_eq!(ast, AST::Concat(vec![AST::Literal('a'), AST::Literal('b')]));
        Ok(())
    }

    #[test]
    fn test_rep_literal() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("a+")?;
        assert_eq!(
            ast,
            AST::Repetition(RepetitionType::OneOrMore, Box::new(AST::Literal('a')))
        );
        Ok(())
    }

    #[test]
    fn test_rep_literal_concat() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("a{1,}b")?;
        assert_eq!(
            ast,
            AST::Concat(vec![
                AST::Repetition(RepetitionType::Lower(1), Box::new(AST::Literal('a'))),
                AST::Literal('b')
            ])
        );
        Ok(())
    }

    #[test]
    fn test_rep_space_literal_concat() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("lots{   4 ,  8      }of ms")?;
        assert_eq!(
            ast,
            AST::Concat(vec![
                AST::Literal('l'),
                AST::Literal('o'),
                AST::Literal('t'),
                AST::Repetition(RepetitionType::Range(4, 8), Box::new(AST::Literal('s'))),
                AST::Literal('o'),
                AST::Literal('f'),
                AST::Literal(' '),
                AST::Literal('m'),
                AST::Literal('s'),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_recursive_rep() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("a{3}*")?;
        assert_eq!(
            ast,
            AST::Repetition(
                RepetitionType::ZeroOrMore,
                Box::new(AST::Repetition(
                    RepetitionType::Exact(3),
                    Box::new(AST::Literal('a'))
                ))
            )
        );
        Ok(())
    }

    #[test]
    fn test_ord_class() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("[abc]")?;
        assert_eq!(
            ast,
            AST::Class(
                false,
                vec![
                    ClassItem::Ordinary('a'),
                    ClassItem::Ordinary('b'),
                    ClassItem::Ordinary('c')
                ]
            )
        );
        Ok(())
    }

    #[test]
    fn test_range_class() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("[A-z]")?;
        assert_eq!(ast, AST::Class(false, vec![ClassItem::Range('A', 'z')]));
        Ok(())
    }

    #[test]
    fn test_neg_class() -> Result<()> {
        let mut parser = Parser::new();
        let ast = parser.parse("[^a-z0-9 ]")?;
        assert_eq!(
            ast,
            AST::Class(
                true,
                vec![
                    ClassItem::Range('a', 'z'),
                    ClassItem::Range('0', '9'),
                    ClassItem::Ordinary(' ')
                ]
            )
        );
        Ok(())
    }
}
