//! Simple Forth Interpreter

use core::fmt::Display;

use alloc::collections::BTreeMap;

use alloc::string::String;
use alloc::vec::Vec;

use crate::print;
use crate::println;
use crate::terminal;

use super::ExitCode;
use super::ShellArgs;


pub enum Value {
    Int(isize),
    Uint(usize),
    String(String),
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Int(x) => write!(f, "{}", x),
            Self::Uint(x) => write!(f, "{}", x),
            Self::String(x) => write!(f, "{}", x),
            Self::Null => write!(f, "NUL"),
        }
    }
}

pub struct ForthInterpreter {
    dictionary: BTreeMap<String, Vec<String>>,
    stack: Vec<Value>,

    compiling: bool,
    current_word: Option<String>,
    current_word_list: Vec<String>,

    heap: [usize; 65536],
}


pub fn main(_args: ShellArgs) -> ExitCode {
    terminal::clear();
    terminal::home();



    ExitCode::Ok
}

impl ForthInterpreter {
    pub fn exec_line(&mut self, line: String) {
        let words: Vec<&str> = line.split_ascii_whitespace().collect();

        for word in words {
            self.exec_word(word)
        }

        println!("ok");
    }

    fn exec_word(&mut self, word: &str) {

        if self.compiling {
            if word.eq_ignore_ascii_case(";") {self.switch_to_interpreting(); return;}

            if self.current_word.is_none() {
                self.current_word = Some(word.into());
            } else {
                self.current_word_list.push(word.into());
            }

            return;
        }

        match word.to_ascii_lowercase().as_str() {
            "." => {self.dot()}
            ":" => {self.switch_to_compilation()},
            ";" => {self.switch_to_interpreting()},

            "!" => {self.write()},
            "@" => {self.read()},
            _ => {
                if self.dictionary.contains_key(word.into()) {
                    for w in self.dictionary[word.into()].clone() {
                        self.exec_word(&w);
                    }
                } else if let Ok(x) = word.parse::<usize>() {
                    self.stack.push(Value::Uint(x));
                } else {
                    println!("Syntax Error: Undefined Word '{}'", word);
                }
            }
        }
    }

    fn switch_to_compilation(&mut self) {
        self.compiling = true;
        self.current_word = None;
        self.current_word_list.clear();
    } 

    fn switch_to_interpreting(&mut self) {
        self.compiling = false;
        self.dictionary.insert(self.current_word.as_ref().unwrap().clone(), self.current_word_list.clone());
    }

    fn read(&mut self) {
        if let Some(address) = self.pop_usize() {
            self.stack.push(Value::Uint(self.heap[address]));
        }
    }

    fn write(&mut self) {
        if let Some(value) = self.pop_usize() {
            if let Some(address) = self.pop_usize() {
                self.heap[address] = value;
            }
        }
    }

    fn pop_usize(&mut self) -> Option<usize> {
        if let Some(value) = self.stack.pop() {
            match value {
                Value::Uint(x) => return Some(x),
                _ => return None,
            }
        } else {
            return None;
        }
    }

    fn dot(&mut self) {
        print!("{}", self.stack.pop().unwrap_or(Value::Null));
    }
}


