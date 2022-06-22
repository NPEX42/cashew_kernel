
use alloc::{vec::Vec, string::{String, ToString}};

use super::{token::*, opcodes::{is_opcode, self}};
pub struct Lexer<'t> {
    text: &'t str,
    current: usize,
    start: usize,
    had_error: bool,

    tokens: Vec<Token>
}

#[allow(dead_code)]
impl<'t> Lexer<'t> {
    pub fn new(input: &'t str) -> Self {
        Self { text: input, current: 0, start: 0, tokens: Vec::new(), had_error: false }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        unsafe {opcodes::init();}
        self.tokens.clear();
        
        while self.has_next() {
            self.primary();
            self.start = self.current;
        }


        self.tokens.clone()
    }

    fn primary(&mut self) {
        if let Some(chr) = self.pop() {
            match chr {
                '[' => self.emit_simple(TokenType::SquareLeft),
                ']' => self.emit_simple(TokenType::SquareRight),
                'A'..='Z' | 'a'..='z' | '_' => self.identifier(),
                '0'..='9' => self.integer(),
                '"' => self.string(),
                ' ' | '\r' | '\t' => {/* no-op */}
                _ => {/* no-op */}
            }
        }
    }

    fn identifier(&mut self) {
        let mut output = self.prev().unwrap().to_string();
        while self.has_next() {
            let curr = self.peek().unwrap();
            if !curr.is_ascii_whitespace() && curr.is_alphanumeric() || curr == '_'  {
                output.push(self.pop().unwrap());
            } else {
                break;
            }


        }

        if self.take(':') {
            self.emit_token(TokenType::Label, output);
        } else if is_opcode(&output) {
            self.emit_token(TokenType::Opcode, output);
        }

    }

    fn integer(&mut self) {
        let mut output = self.prev().unwrap().to_string();
        while self.has_next() && self.peek().unwrap().is_ascii_digit() {
            output.push(self.pop().unwrap());
        }

        self.emit_token(TokenType::Integer, output);
    }

    fn string(&mut self) {
        let mut output = String::new();
        while self.has_next() && self.peek().unwrap() != '"' {
            output.push(self.pop().unwrap());
        }

        self.pop();

        self.emit_token(TokenType::String, output);
    }

    fn take(&mut self, target: char) -> bool {
        if self.has_next() {
            return self.peek().unwrap() == target;
        }
        false
    }

    fn emit_token(&mut self, kind: TokenType, lexeme: String) {
        self.tokens.push(Token::new(kind, lexeme))
    }

    fn emit_simple(&mut self, kind: TokenType) {
        self.tokens.push(Token::new(kind, "".into()))
    }

    fn lexeme_str(&mut self) -> &'t str {
        let lexeme = &self.text[self.start+1..self.current];
        lexeme
    }

    fn lexeme(&mut self) -> &'t str {
        let lexeme = &self.text[self.start..=self.current];
        lexeme
    }

    fn lookahead(&self, n: usize) -> Option<char> {
        if self.text.len() > self.current + n {
            self.text.chars().nth(self.current + n)
        } else {
            None
        }
    }

    fn lookback(&self, n: usize) -> Option<char> {
        return self.text.chars().nth(self.current.saturating_sub(n));
    }

    fn prev(&self) -> Option<char> {
        return self.text.chars().nth(self.current.saturating_sub(1))
    }

    fn pop(&mut self) -> Option<char> {
        if self.has_next() {
            let res = self.text.chars().nth(self.current);
            self.current += 1;
            res
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        if self.has_next() {
            self.text.chars().nth(self.current)
        } else {
            None
        }
    }

    fn has_next(&self) -> bool {
        self.text.len() > self.current
    }
}