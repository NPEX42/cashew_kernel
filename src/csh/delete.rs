use crate::{println, csh::ErrorCode, vfs::drivers::simple_fat};

use super::{ShellArgs, ExitCode};

pub fn main(args: ShellArgs) -> ExitCode {

    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return ExitCode::Error(ErrorCode::Usage)
    }

    simple_fat::delete_file(&args[1]);

    ExitCode::Ok
}