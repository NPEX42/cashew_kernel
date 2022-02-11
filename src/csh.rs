use alloc::{vec::Vec, string::{String, ToString}, collections::BTreeMap};

use crate::{data::{hashmap::HashMap}, input, sprint, mem, println};

pub mod ls;
pub mod cat;
pub mod objdump;

pub type ShellArgs = Vec<String>;
pub type ProgramMain = fn(ShellArgs) -> ExitCode;
static mut PROGS: BTreeMap<String, ProgramMain> = BTreeMap::new();

pub fn add_program(name: &str, main: ProgramMain) -> Result<(), ()> {
    unsafe {
        PROGS.insert(name.into(), main);
    }

    Ok(())
}

pub fn init() -> Result<(), ()> {
    add_program("ls", ls::main)?;
    add_program("cat", cat::main)?;
    add_program("csh", main)?;
    add_program("mem", mem::csh_stats)?;

    add_program("objdump", objdump::main)?;

    Ok(())
}


#[derive(Debug)]
pub enum ExitCode {
    Ok,
    Error(ErrorCode)
}

#[derive(Debug)]
pub enum ErrorCode {
    Other(u8),

    General,
    MisuseBuiltin,
    CommandCannotExecute,
    CommandNotFound,
    InvalidArgument,

    FatalError(u8),
    TerminatedCtlC,
    Usage
}

impl ExitCode {
    pub fn unix(&self) -> u8 {
        match &self {
            &Self::Ok => 0,
            &Self::Error(code) => code.unix(),
        }
    }
}

impl ErrorCode {
    pub fn unix(&self) -> u8 {
        match &self {
            ErrorCode::Other(c) => *c,
            ErrorCode::General => 1,
            ErrorCode::MisuseBuiltin => 2,
            ErrorCode::CommandCannotExecute => 126,
            ErrorCode::CommandNotFound => 127,
            ErrorCode::InvalidArgument => 128,
            ErrorCode::FatalError(sig) => 128 + ((*sig) % 9),
            ErrorCode::TerminatedCtlC => 130,
            ErrorCode::Usage => 10,
        }
    }
}


pub fn exec(cmd: &str) -> ExitCode {
    if cmd.is_empty() {return ExitCode::Ok}
    let parts: ShellArgs = cmd.to_string().split_ascii_whitespace().map(|s| {s.to_string()}).collect::<Vec<String>>();
    let ec = if let Some(main) = unsafe { PROGS.get(&parts[0]) } {
        main(parts)
    } else {
        sprint!("No Such Program - '{}'\n", parts[0]);
        ExitCode::Error(ErrorCode::CommandNotFound)
    };
    ec
}

pub fn main(_: ShellArgs) -> ExitCode {

    let mut line = String::new();
    while line != "exit".to_string() {
        line = input::prompt(">> ");

        match exec(&line) {
            ExitCode::Error(ec) => {println!("Command Returned Code {} ({0:#x} - {0:?})", ec.unix());}
            ExitCode::Ok => {}
        }
    }

    ExitCode::Ok
}