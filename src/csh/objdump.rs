use crate::{println, vfs::drivers::ustar::FileInfo};

use super::*;
use elf_rs::*;

pub fn main(args: ShellArgs) -> ExitCode {
    if args.len() < 2 {println!("Usage: {} [filepath]", args[0]); return ExitCode::Error(ErrorCode::Usage);}
    let filepath = &args[1];
    if let Ok(file) = FileInfo::open(filepath) {
        let bytes = file.to_vec();
        let elf = Elf::from_bytes(&bytes).expect("Failed To Parse ELF File.");
        for section in elf.section_header_iter() {
            println!("Section: '{}' - VADDR 0x{:04x} .. 0x{:04x} - Align: {} - Flags: {:?}",
                String::from_utf8(section.section_name().to_vec()).expect("Failed To Get Section Name"), 
                section.addr(), section.addr() + section.size(), section.addralign(), section.flags()
            );
        }
    }
    ExitCode::Ok
}