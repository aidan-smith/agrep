use std::collections::HashSet;

use crate::{regex::{self, Regex, RepetitionType}, ast};

#[derive(Debug)]
pub struct Transition {
    next: usize,
    input: Option<(char, char)>,
}

#[derive(Debug)]
pub struct State {
    transitions: Vec<Transition>,
}

type StateID = usize;
const ZERO: StateID = 0;
const FINAL: StateID = usize::MAX;

// impl StateID {
// }

#[derive(Debug)]
pub struct NFA {
    states: Vec<State>,
    initial: StateID,
    accepting: StateID,
}

impl NFA {
    fn new() -> Self {
        Self {
            states: Vec::new(),
            initial: ZERO,
            accepting: FINAL,
        }
    }

    pub fn from_regex(regex: &Regex) -> Self {
        let mut nfa = Self::new();
        NFABuilder::new(&mut nfa, regex).build();
        nfa
    }

    fn add_state(&mut self) -> usize {
        let state = State {
            transitions: Vec::new(),
        };
        self.states.push(state);
        self.states.len() - 1
    }

    fn add_epsilon_transition(&mut self, from: usize, to: usize) {
        self.states[from].transitions.push(Transition {
            next: to,
            input: None,
        });
    }

    fn add_char_transition(&mut self, from: usize, to: usize, input: char) {
        self.states[from].transitions.push(Transition {
            next: to,
            input: Some((input, input)),
        });
    }

    fn add_range_transition(&mut self, from: usize, to: usize, start: char, end: char) {
        self.states[from].transitions.push(Transition {
            next: to,
            input: Some((start, end)),
        });
    }
}

struct Component {
    initial: usize,
    accepting: usize,
}

struct NFABuilder<'a> {
    nfa: &'a mut NFA,
    regex: &'a Regex,
}

impl<'a> NFABuilder<'a> {
    fn new(nfa: &'a mut NFA, regex: &'a Regex) -> Self {
        Self { nfa, regex }
    }

    fn build_empty(&mut self) -> Component {
        let initial = self.nfa.add_state();
        let accepting = self.nfa.add_state();
        self.nfa.add_epsilon_transition(initial, accepting);
        Component { initial, accepting }
    }

    fn build_literal(&mut self, input: &[char]) -> Component {
        let initial = self.nfa.add_state();
        let accepting = self.nfa.add_state();
        self.nfa.add_char_transition(initial, accepting, input[0]);
        Component { initial, accepting }
    }

    // TODO: Support negated classes.
    fn build_class(&mut self, _negated: bool, items: Vec<ast::ClassItem>) -> Component {
        let initial = self.nfa.add_state();
        let accepting = self.nfa.add_state();
        for item in items {
            match item {
                ast::ClassItem::Ordinary(literal) => {
                    self.nfa.add_char_transition(initial, accepting, literal);
                }
                ast::ClassItem::Range{start, end} => {
                    self.nfa.add_range_transition(initial, accepting, start, end);
                }
                _ => unimplemented!(),
            }
        }
        Component { initial, accepting }
    }

    // Note: This should be changed when we add support for starting in the middle of a string.
    fn build_assert(&mut self, anchor_type: &ast::AnchorType) -> Component {
        let initial = self.nfa.add_state();
        let accepting = self.nfa.add_state();
        match anchor_type {
            ast::AnchorType::LineStart => {
                self.nfa.add_epsilon_transition(initial, accepting);
            }
            ast::AnchorType::LineEnd => {
                self.nfa.add_epsilon_transition(initial, accepting);
            }
        }
        Component { initial, accepting }
    }

    fn build_repetition(&mut self, repetition_type: RepetitionType, regex: &Regex) -> Component {
        let initial = self.nfa.add_state();
        let accepting = self.nfa.add_state();
        match repetition_type {
            RepetitionType::Exact(n) => {
                let mut prev = initial;
                for _ in 0..n {
                    let comp = self.build_node(regex);
                    self.nfa.add_epsilon_transition(prev, comp.initial);
                    prev = comp.accepting;
                }
                self.nfa.add_epsilon_transition(prev, accepting);
            }
            RepetitionType::Lower(n) => {
                let mut prev = initial;
                for _ in 0..n {
                    let comp = self.build_node(regex);
                    self.nfa.add_epsilon_transition(prev, comp.initial);
                    prev = comp.accepting;
                }
                let comp = self.build_node(regex);
                self.nfa.add_epsilon_transition(prev, comp.initial);
                self.nfa.add_epsilon_transition(prev, accepting);
                self.nfa.add_epsilon_transition(comp.accepting, comp.initial);
            }
            RepetitionType::Range(min, max) => {
                let mut prev = initial;
                for _ in 0..min {
                    let comp = self.build_node(regex);
                    self.nfa.add_epsilon_transition(prev, comp.initial);
                    prev = comp.accepting;
                }
                self.nfa.add_epsilon_transition(prev, accepting);
                for _ in min..max {
                    let comp = self.build_node(regex);
                    self.nfa.add_epsilon_transition(prev, comp.initial);
                    self.nfa.add_epsilon_transition(comp.accepting, accepting);
                    prev = comp.accepting;
                }
            }
        }
        Component { initial, accepting }
    }

    fn build_concat(&mut self, regexes: &Vec<Regex>) -> Component {
        let initial = self.nfa.add_state();
        let mut prev = initial;
        for regex in regexes {
            let comp = self.build_node(regex);
            self.nfa.add_epsilon_transition(prev, comp.initial);
            prev = comp.accepting;
        }
        Component {
            initial,
            accepting: prev,
        }
    }

    fn build_alternation(&mut self, regexes: &Vec<Regex>) -> Component {
        let initial = self.nfa.add_state();
        let accepting = self.nfa.add_state();
        for regex in regexes {
            let comp = self.build_node(regex);
            self.nfa.add_epsilon_transition(initial, comp.initial);
            self.nfa.add_epsilon_transition(comp.accepting, accepting);
        }
        Component { initial, accepting }
    }

    fn build_node(&mut self, regex: &Regex) -> Component {
        match regex {
            Regex::Empty => self.build_empty(),
            Regex::Literal(input) => self.build_literal(input),
            Regex::Class { negated, items } => self.build_class(*negated, items.clone()),
            Regex::Assert(anchor_type) => self.build_assert(&anchor_type),
            Regex::Repetition(repetition_type, regex) => {
                self.build_repetition(repetition_type.clone(), regex)
            }
            Regex::Concat(regexes) => self.build_concat(regexes),
            Regex::Alternation(regexes) => self.build_alternation(regexes),
        }
    }

    fn build(&mut self) {
        let comp = self.build_node(self.regex);
        self.nfa.initial = comp.initial;
        self.nfa.accepting = comp.accepting;
    }
}

struct NFAVM<'a> {
    nfa: &'a NFA,
    input: &'a [char],
    pos: usize,
    state: StateID,
}

impl<'a> NFAVM<'a> {
    pub fn new(nfa: &'a NFA, input: &'a [char]) -> Self {
        Self {
            nfa,
            input,
            pos: 0,
            state: nfa.initial,
        }
    }

    fn step(&mut self) -> bool {
        let mut next_state = None;
        for transition in &self.nfa.states[self.state].transitions {
            match transition.input {
                None => {
                    next_state = Some(transition.next);
                    break;
                }
                Some((start, end)) => {
                    if start <= self.input[self.pos] && self.input[self.pos] <= end {
                        next_state = Some(transition.next);
                        break;
                    }
                }
            }
        }
        if let Some(next_state) = next_state {
            self.state = next_state;
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn run(&mut self) -> bool {
        while self.pos < self.input.len() {
            if !self.step() {
                return false;
            }
        }
        true
    }
}
