use alloc::string::String;


#[derive(Debug, Clone,PartialEq, PartialOrd)]
pub enum TokenType {
    SquareRight, SquareLeft, Register, Opcode, Integer, String, Char, Label, 
    Identifier, 
}
#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenType,
    lexeme: String,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String) -> Self {
        Self {
            kind,
            lexeme
        }
    }

    pub fn lexeme(&self) -> &String {
        &self.lexeme
    }

    pub fn kind(&self) -> &TokenType {
        &self.kind
    }
}

impl PartialEq<Token> for TokenType {
    fn eq(&self, other: &Token) -> bool {
        self == &other.kind
    }
}