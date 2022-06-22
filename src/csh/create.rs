use crate::{println, csh::ErrorCode, vfs::drivers::{simple_fat, FileIO}};

use super::{ShellArgs, ExitCode};

pub fn main(args: ShellArgs) -> ExitCode {

    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return ExitCode::Error(ErrorCode::Usage)
    }

    if let Some(mut file) = simple_fat::create_file(&args[1]) {
        println!("Created File: {}", file.name());
        file.close();
    } else {
        return ExitCode::Error(ErrorCode::FatalError(10))
    }

    ExitCode::Ok
}