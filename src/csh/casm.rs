//! CASM - Cashew Assembler
use self::lexer::Lexer;
use super::*;

pub mod lexer;
pub mod parser;
pub mod token;
pub mod opcodes;

const VERSION_MAJOR: usize = 0;
const VERSION_MINOR: usize = 1;


pub fn main(args: ShellArgs) -> ExitCode {
    if !args.contains(&String::from("-q")) {
        println!("CASM v{}.{}", VERSION_MAJOR, VERSION_MINOR);
    }



    let mut lexer = Lexer::new("_Start: pushi 10\n pushi 20\n add");
    let tokens = lexer.tokenize();
    for token in tokens {
        println!(" - {:?}({})",token.kind(), token.lexeme());
    }

    
    ExitCode::Ok
}
