use std::collections::HashMap;
use std::collections::HashSet;
use std::env;

type Label = Option<char>;
type State = usize;
type StateSet = HashSet<State>;
type Transitions = HashMap<State, Vec<(Label, State)>>;

#[derive(Debug)]
pub struct Nfa {
    start: State,
    accept: State,
    transitions: Transitions,
}

impl Nfa {
    fn new(start: State, accept: State, transitions: Transitions) -> Self {
        Self {
            start,
            accept,
            transitions,
        }
    }

    fn add_transition(&mut self, from: State, label: Label, to: State) {
        self.transitions.entry(from).or_default().push((label, to));
    }

    fn eps_closure(&self, states: StateSet) -> StateSet {

        let mut ec = states;
        let mut stack: Vec<State> = ec.clone().into_iter().collect();

        while let Some(state) = stack.pop() {
            if let Some(pairs) = self.transitions.get(&state) {
                for (label, s) in pairs.iter() {
                    if label.is_none() && ec.insert(*s) {
                        stack.push(*s);
                    }
                }
            }
        }
        ec
    }

    fn get_move(&self, states: &StateSet, symbol: char) -> StateSet {

        let mut res = StateSet::new();

        for from in states.iter() {
            if let Some(pairs) = self.transitions.get(from) {
                for (label, to) in pairs.iter() {
                    if let Some(c) = label {
                        if *c == symbol {
                            res.insert(*to);
                        }
                    }
                }
            }
        }
        res
    }

    fn simulate(&self, input: &str) -> bool {
        // From figure 3.37 of dragon book
        let s0 = HashSet::from([self.start]);
        let mut states = self.eps_closure(s0);

        for c in input.chars() {
            let from_move = self.get_move(&states, c);
            states = self.eps_closure(from_move);
        }

        return states.contains(&self.accept);
    }
}

struct Parser<'a> {
    s: &'a str, // pattern
    i: usize, // index
    next_state: State,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            s: input,
            i: 0,
            next_state: 0,
        }   
    }

    fn fresh(&mut self) -> State {
        let state = self.next_state;
        self.next_state += 1;
        state
    }

    fn peek(&self) -> Label {
        self.s[self.i..].chars().next()
    }

    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.consume();
            true
        } else {
            false
        }
    }

    fn consume(&mut self) -> Label {
        let c = self.peek()?;
        self.i += c.len_utf8();
        Some(c)
    }

    fn parse_regex(&mut self) -> Result<Nfa, String> {
        self.parse_alt()
    }

    fn parse_alt(&mut self) -> Result<Nfa, String> {
        let mut left = self.parse_concat()?;
        while self.eat('|') {
            let right = self.parse_concat()?;
            left = self.build_alt(left, right);
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Nfa, String> {

        let mut res: Option<Nfa> = None;

        while let Some(c) = self.peek() {
            if c == ')' || c == '|' {
                break;
            }
            let rep = self.parse_rep()?;
            res = match res {
                Some(nfa) => Some(self.build_concat(nfa, rep)),
                None => Some(rep),
            }
        }

        match res {
            None => Err("Expected expression".into()),
            Some(nfa) => Ok(nfa),
        }
    }

    fn parse_rep(&mut self) -> Result<Nfa, String> {
        let mut nfa = self.parse_primary()?;
        if self.eat('*') {
            nfa = self.build_star(nfa);
        }
        Ok(nfa)
    }

    fn parse_primary(&mut self) -> Result<Nfa, String> {
        match self.peek() {
            Some('(') => {
                self.consume();
                let nfa = self.parse_regex()?;
                if !self.eat(')') {
                    return Err("Expected ')'".into());
                }
                Ok(nfa)
            }
            Some('*') | Some('|') | Some(')') | None => {
                Err("Unexpected token".into())
            }
            Some(c) => {
                self.consume();
                Ok(self.build_char(c))
            }
        }
    }

    fn build_char(&mut self, c: char) -> Nfa {
        let s = self.fresh();
        let a = self.fresh();
        let mut nfa = Nfa::new(s, a, Transitions::new());
        nfa.add_transition(s, Some(c), a);
        nfa
    }

    fn build_concat(&mut self, left: Nfa, right: Nfa) -> Nfa {
        let mut transitions = left.transitions;
        transitions.extend(right.transitions);
        let mut nfa = Nfa::new(left. start, right.accept, transitions);
        nfa.add_transition(left.accept, None, right.start);
        nfa
    }

    fn build_alt(&mut self, left: Nfa, right: Nfa) -> Nfa {
        let s = self.fresh();
        let a = self.fresh();
        let mut transitions = left.transitions;
        transitions.extend(right.transitions);
        let mut nfa = Nfa::new(s, a, transitions);
        nfa.add_transition(s, None, left.start);
        nfa.add_transition(s, None, right.start);
        nfa.add_transition(left.accept, None, a);
        nfa.add_transition(right.accept, None, a);
        nfa
    }

    fn build_star(&mut self, inner: Nfa) -> Nfa {
        let s = self.fresh();
        let a = self.fresh();
        let mut nfa = Nfa::new(s, a, inner.transitions);
        nfa.add_transition(s, None, inner.start);
        nfa.add_transition(s, None, a);
        nfa.add_transition(inner.accept, None, inner.start);
        nfa.add_transition(inner.accept, None, a);
        nfa
    }
}

pub fn regex_to_nfa(pattern: &str) -> Result<Nfa, String> {
    let mut p = Parser::new(pattern);
    let nfa = p.parse_regex()?;
    if p.peek().is_some() {
        return Err("Unexpected trailing characters".into());
    }
    Ok(nfa)
}

pub fn regex_match(pattern: &str, input: &str) -> bool {
    let nfa = regex_to_nfa(pattern).unwrap();
    nfa.simulate(input)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: cargo run -- <pattern> <input>");
        std::process::exit(0);
    }

    let pattern = &args[1];
    let input = &args[2];

    if regex_match(pattern, input) {
        println!("{} matches {}", input, pattern);
    } else {
        println!("{} doesn't match {}", input, pattern);
    }
}
