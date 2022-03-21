use crate::vfs::drivers::csh_fat::FAT;

use super::*;

pub fn main(_args: ShellArgs) -> ExitCode {
    for i in 0..16 * FAT::entry_count() {
        if let Ok(file) = FAT::get_entry(i) {
            println!(
                "entry: {} - {} - {}B - {}",
                i, file.name, file.size, file.data_start
            );
        }
    }

    ExitCode::Ok
}
