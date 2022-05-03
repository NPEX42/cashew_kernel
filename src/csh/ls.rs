use crate::{api::fs::csh_fat::active_dir};

use super::*;

pub fn main(_args: ShellArgs) -> ExitCode {
    match active_dir() {
        Some(node) => {
            println!("Current: {} - {} Entries", node.name(), node.children().len());

            for child in node.children() {
                println!("- {} | {} Bytes", child.name(), child.size());
            }
        },
        None => {println!("No Dir Mounted");},
    }

    ExitCode::Ok
}
