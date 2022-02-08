use alloc::{vec::Vec, string::{String, ToString}};

use crate::{data::{hashmap::HashMap}, input, sprint};

pub mod ls;
pub mod cat;

pub type ShellArgs = Vec<String>;
pub type ProgramMain = fn(ShellArgs) -> ExitCode;
static mut PROGS: HashMap<ProgramMain> = HashMap::new();

pub fn add_program(name: &str, main: ProgramMain) {
    unsafe {
        PROGS.insert(&name.into(), main);
    }
}

pub fn init() {
    add_program("ls", ls::main);
    add_program("cat", cat::main);
    add_program("csh", main);
}


pub enum ExitCode {
    Ok,
    Error(isize)
}


pub fn exec(cmd: &str) -> ExitCode {
    if cmd.is_empty() {return ExitCode::Ok;}
    let parts: ShellArgs = cmd.to_string().split_ascii_whitespace().map(|s| {s.to_string()}).collect::<Vec<String>>();
    let ec = if let Some(main) = unsafe { PROGS.get(&parts[0]) } {
        main(parts)
    } else {
        sprint!("No Such Program - '{}'\n", parts[0]);
        ExitCode::Error(0)
    };
    ec
}

pub fn main(_: ShellArgs) -> ExitCode {

    let mut line = String::new();
    while line != "exit".to_string() {
        line = input::prompt(">> ");

        exec(&line);
    }

    ExitCode::Ok
}