use crate::{println, vfs::drivers::ustar::FileInfo};

use super::*;
pub fn main(args: ShellArgs) -> ExitCode {

    if args.len() < 2 {println!("Usage: {} <filepath>", args[0]); return ExitCode::Error(ErrorCode::Usage)}
    let path = &args[1];
    if let Ok(file) = FileInfo::open(path) {
        println!("{}", file);
    } else {
        println!("Error: File Not Found: '{}'", path);
        return ExitCode::Error(ErrorCode::General);
    }

    ExitCode::Ok
}