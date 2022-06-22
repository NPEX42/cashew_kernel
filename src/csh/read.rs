use crate::{println, csh::ErrorCode, vfs::drivers::{simple_fat, FileRead}, device::stdout};

use super::{ShellArgs, ExitCode};

pub fn main(args: ShellArgs) -> ExitCode {

    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return ExitCode::Error(ErrorCode::Usage)
    }

    match simple_fat::load_file(&args[1]) {
        None => {return ExitCode::Error(ErrorCode::FatalError(1))},
        Some(file) => {
            stdout::println!("{}", file.read_to_string());
            return ExitCode::Ok;
        }
    }
}