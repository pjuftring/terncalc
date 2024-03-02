#![no_std]
extern crate alloc;
mod cppalloc;

use alloc::{boxed::Box, rc::Rc};
use core::{
    ffi::{c_char, c_int, c_uchar},
    ptr::null,
};

type N = i64;

#[derive(Clone, Copy)]
enum Operator {
    Plus,
    Minus,
    Times,
    Div,
}

#[derive(Clone, Copy)]
enum Parenthesis {
    Open,
    Close,
}

#[derive(Clone, Copy)]
enum Input {
    Number(u8),
    Operator(Operator),
    Parenthesis(Parenthesis),
    Equals,
    Clear,
    ClearAll,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StateID {
    Begin,
    BeginAfterOperator,
    BeginAfterEquals,
    Number,
    ForceOperator,
}

#[derive(Clone, PartialEq, Eq)]
struct State {
    sum: N,
    factor: N,
    division: bool,
    // Normally, we present "current" but after Input::Equals
    // we want to present "sum". We save this in order to present
    // the correct value after "undo" or "redo"
    present_sum: bool,
    current: N,
    id: StateID,
    stack: Option<Rc<State>>,
}

impl State {
    fn new() -> State {
        State {
            sum: 0,
            factor: 1,
            division: false,
            present_sum: false,
            current: 0,
            id: StateID::Begin,
            stack: None,
        }
    }
    fn finish_factor(&mut self) -> Result<(), &'static str> {
        if self.division {
            self.factor = self
                .factor
                .checked_div(self.current)
                .ok_or("Division by zero\0")?;
            self.division = false;
        } else {
            self.factor = self
                .factor
                .checked_mul(self.current)
                .ok_or("Multiplication overflow\0")?;
        }
        self.current = 0;
        Ok(())
    }
    fn finish_current(&mut self) -> Result<(), &'static str> {
        self.finish_factor()?;
        self.sum = self
            .sum
            .checked_add(self.factor)
            .ok_or("Addition overflow\0")?;
        self.factor = 1;
        Ok(())
    }
    fn number(&self, n: N, after_equals: bool) -> Result<State, &'static str> {
        let mut state = if after_equals {
            Self::new()
        } else {
            self.clone()
        };
        if !(0..=3).contains(&n) {
            return Err("Nonternary input\0");
        }
        state.current = state
            .current
            .checked_mul(3)
            .ok_or("Multiplication overflow\0")?
            .checked_add(n)
            .ok_or("Addition overflow\0")?;
        state.present_sum = false;
        state.id = StateID::Number;

        // Check if there is an overflow if this number is applied
        let mut state2 = state.clone();
        state2.finish_current()?;

        Ok(state)
    }
    fn plus(&self, minus: bool) -> Result<State, &'static str> {
        let mut state = self.clone();
        state.finish_current()?;
        if minus {
            state.factor = -1;
        }
        state.present_sum = false;
        state.id = StateID::BeginAfterOperator;
        Ok(state)
    }
    fn times(&self, after_equals: bool, division: bool) -> Result<State, &'static str> {
        let mut state = self.clone();
        if after_equals {
            state.factor = state.sum;
            state.sum = 0;
        } else {
            state.finish_factor()?;
        }
        state.division = division;
        state.present_sum = false;
        state.id = StateID::Begin;
        Ok(state)
    }
    fn open(this: &Rc<Self>, after_equals: bool) -> State {
        let mut state = State::new();
        if after_equals {
            state.stack = Some(Rc::new(State::new()));
        } else {
            state.stack = Some(Rc::clone(this));
        }
        state.present_sum = false;
        state.id = StateID::Begin;
        state
    }
    fn close(&self) -> Result<State, &'static str> {
        let mut state = self.clone();
        state.finish_current()?;
        let value = state.sum;

        let mut state =
            Self::clone(&state.stack.ok_or("No matching open parenthesis\0")? as &State);
        state.current = value;

        // After closing a parenthesis, we cannot input anything anymore
        // Therefore, we have to check already here if our output does not
        // lead to divisions by zero.
        let mut state2 = state.clone();
        state2.finish_current()?;

        state.present_sum = false;
        state.id = StateID::ForceOperator;
        Ok(state)
    }
    fn clear(&self) -> State {
        let mut state = self.clone();
        state.current = 0;
        state.present_sum = false;
        state.id = StateID::Number;
        state
    }
    fn equals(&self) -> Result<State, &'static str> {
        let mut state = self.clone();
        state.finish_current()?;
        state.present_sum = true;
        state.id = StateID::BeginAfterEquals;
        Ok(state)
    }
    fn step(this: &Rc<Self>, input: Input) -> Result<Rc<Self>, &'static str> {
        match input {
            Input::Number(n) => match this.id {
                StateID::Begin | StateID::BeginAfterOperator | StateID::Number => {
                    Ok(Rc::new(this.number(n as _, false)?))
                }
                StateID::BeginAfterEquals => Ok(Rc::new(this.number(n as _, true)?)),
                StateID::ForceOperator => Err("Expect operator\0"),
            },
            Input::Operator(Operator::Plus) => match this.id {
                StateID::BeginAfterEquals | StateID::Number | StateID::ForceOperator => {
                    Ok(Rc::new(this.plus(false)?))
                }
                StateID::Begin | StateID::BeginAfterOperator => Err("Expect number\0"),
            },
            Input::Operator(Operator::Minus) => match this.id {
                StateID::Begin
                | StateID::BeginAfterEquals
                | StateID::Number
                | StateID::ForceOperator => Ok(Rc::new(this.plus(true)?)),
                StateID::BeginAfterOperator => Err("Expect number (without minus)\0"),
            },
            Input::Operator(op) => {
                let division = match op {
                    Operator::Plus | Operator::Minus => unreachable!(),
                    Operator::Times => false,
                    Operator::Div => true,
                };
                match this.id {
                    StateID::Number | StateID::ForceOperator => {
                        Ok(Rc::new(this.times(false, division)?))
                    }
                    StateID::BeginAfterEquals => Ok(Rc::new(this.times(true, division)?)),
                    StateID::Begin | StateID::BeginAfterOperator => Err("Expect number\0"),
                }
            }
            Input::Parenthesis(Parenthesis::Open) => match this.id {
                StateID::Begin | StateID::BeginAfterOperator => {
                    Ok(Rc::new(Self::open(this, false)))
                }
                StateID::BeginAfterEquals => Ok(Rc::new(Self::open(this, true))),
                StateID::Number | StateID::ForceOperator => Err("Expect number or parenthesis\0"),
            },
            Input::Parenthesis(Parenthesis::Close) => match this.id {
                StateID::Number => this.close().map(|state| Rc::new(state)),
                StateID::Begin
                | StateID::BeginAfterOperator
                | StateID::BeginAfterEquals
                | StateID::ForceOperator => Err("Expect operator\0"),
            },
            Input::Equals => {
                if this.stack.is_some() {
                    Err("Unclosed parenthesis\0")
                } else {
                    match this.id {
                        StateID::Begin
                        | StateID::BeginAfterOperator
                        | StateID::Number
                        | StateID::ForceOperator => Ok(Rc::new(this.equals()?)),
                        StateID::BeginAfterEquals => Err("Expect input\0"),
                    }
                }
            }
            Input::Clear => {
                if this.current == 0 {
                    Err("Expect input\0")
                } else {
                    Ok(Rc::new(this.clear()))
                }
            }
            Input::ClearAll => {
                if Self::eq(this, &Self::new()) {
                    Err("Expect input\0")
                } else {
                    Ok(Rc::new(Self::new()))
                }
            }
        }
    }
}

const BUFFER_SIZE: usize = 64;

pub struct Terncalc {
    states: [Option<(usize, Rc<State>)>; BUFFER_SIZE],
    pos: usize,
    count: usize,
}

impl Terncalc {
    pub fn new() -> Terncalc {
        let state = Rc::new(State::new());
        const NONE: Option<(usize, Rc<State>)> = None;
        let mut states = [NONE; 64];
        states[0] = Some((0, state));
        Terncalc {
            states,
            pos: 0,
            count: 1,
        }
    }
    fn next_pos(pos: usize) -> usize {
        if pos < BUFFER_SIZE - 1 {
            pos + 1
        } else {
            0
        }
    }
    fn previous_pos(pos: usize) -> usize {
        if pos > 0 {
            pos - 1
        } else {
            BUFFER_SIZE - 1
        }
    }
    fn current_id_and_state(&self) -> (usize, Rc<State>) {
        if let Some((id, state)) = &self.states[self.pos] {
            (*id, state.clone())
        } else {
            unreachable!()
        }
    }
    fn next_state(&mut self, state: Rc<State>) {
        self.pos = Self::next_pos(self.pos);
        self.states[self.pos] = Some((self.count, state));
        self.count += 1;
    }
    fn force_step(&mut self, input: Input) {
        let state = self.current_id_and_state().1;
        let next = State::step(&state, input).unwrap();
        self.next_state(next);
    }
    fn undo_enabled(&self) -> *const c_char {
        let current = self.current_id_and_state().0;
        if let Some((previous, _)) = &self.states[Self::previous_pos(self.pos)] {
            if *previous > current {
                "Undo buffer exhausted\0".as_ptr() as _
            } else {
                null()
            }
        } else {
            "No previous calculation\0".as_ptr() as _
        }
    }
    fn redo_enabled(&self) -> *const c_char {
        let current = self.current_id_and_state().0;
        if let Some((next, _)) = &self.states[Self::next_pos(self.pos)] {
            if *next < current {
                "No next calculation\0".as_ptr() as _
            } else {
                null()
            }
        } else {
            "No next calculation\0".as_ptr() as _
        }
    }
    fn force_undo(&mut self) {
        self.pos = Self::previous_pos(self.pos);
    }
    fn force_redo(&mut self) {
        self.pos = Self::next_pos(self.pos);
    }
    pub fn input(&mut self, input_char: c_uchar) {
        if input_char == b'U' {
            self.force_undo();
        } else if input_char == b'R' {
            self.force_redo();
        } else {
            let input = match input_char as _ {
                b'0' => Input::Number(0),
                b'1' => Input::Number(1),
                b'2' => Input::Number(2),
                b'+' => Input::Operator(Operator::Plus),
                b'-' => Input::Operator(Operator::Minus),
                b'*' => Input::Operator(Operator::Times),
                b'/' => Input::Operator(Operator::Div),
                b'(' => Input::Parenthesis(Parenthesis::Open),
                b')' => Input::Parenthesis(Parenthesis::Close),
                b'=' => Input::Equals,
                b'C' => Input::Clear,
                b'A' => Input::ClearAll,
                _ => unreachable!(),
            };
            self.force_step(input);
        }
    }
}

#[no_mangle]
pub extern "C" fn calc_new() -> *mut Terncalc {
    let calc = Box::new(Terncalc::new());
    Box::leak(calc)
}

#[no_mangle]
pub extern "C" fn calc_drop(calc: *mut Terncalc) {
    let _ = unsafe { Box::from_raw(calc) };
    // implicitely dropped
}

#[no_mangle]
pub extern "C" fn calc_input(calc: &mut Terncalc, input: c_uchar) -> i64 {
    calc.input(input);
    let state = calc.current_id_and_state().1;
    if state.present_sum {
        state.sum
    } else {
        state.current
    }
}

#[no_mangle]
pub extern "C" fn calc_enabled(calc: &mut Terncalc, vector: &mut [*const c_char; 14]) {
    for (n, input) in [
        Input::Number(0),
        Input::Number(1),
        Input::Number(2),
        Input::Operator(Operator::Plus),
        Input::Operator(Operator::Minus),
        Input::Operator(Operator::Times),
        Input::Operator(Operator::Div),
        Input::Parenthesis(Parenthesis::Open),
        Input::Parenthesis(Parenthesis::Close),
        Input::Equals,
        Input::Clear,
        Input::ClearAll,
    ]
    .into_iter()
    .enumerate()
    {
        vector[n] = match State::step(&calc.current_id_and_state().1, input) {
            Ok(_) => null(),
            Err(text) => text.as_ptr() as _,
        }
    }
    vector[12] = calc.undo_enabled();
    vector[13] = calc.redo_enabled();
}

#[no_mangle]
pub extern "C" fn number_to_text(chars: &mut [c_char; 64], mut number: i64) -> c_int {
    // 64bit numbers in ternary representation cannot exceed 41(?) characters.
    // With '-', we require 42. So, let's take 64 to be safe.
    chars[63] = 0; // terminating null
    if number == 0 {
        chars[62] = b'0' as _;
        return 62;
    }
    let minus = number < 0;
    number = number.abs();

    let mut pos = 63;
    while number > 0 {
        pos -= 1;
        chars[pos] = b'0' as c_char + (number % 3) as c_char;
        number /= 3;
    }
    if minus {
        pos -= 1;
        chars[pos] = b'-' as _;
    }
    pos as _
}
