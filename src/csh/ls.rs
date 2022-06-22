
use crate::vfs::drivers::simple_fat::list;

use super::*;

pub fn main(_args: ShellArgs) -> ExitCode {
    for file in &list() {
        println!("{} - {} Bytes", file.name(), file.size_on_disk());
    }
    ExitCode::Ok
}
