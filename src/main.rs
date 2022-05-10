#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(cashew_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use core::arch::asm;




use alloc::{vec::Vec, string::String};
#[cfg(not(test))]
use bootloader::entry_point;
use bootloader::BootInfo;
use cashew_kernel::{ata, graphics_2d::*, kerr, println, csh, vfs};
use elf_rs::{ElfFile};

#[cfg(not(test))]
entry_point!(kernel_main);

static mut FRAME: Frame = Frame::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(mut fb) = boot_info.framebuffer.as_mut() {
        cashew_kernel::boot(boot_info);


        println!("Booting Complete, Press Any Key To continue");

        match vfs::open_file("initrd/bin/a.out") {
            Some(file) => {
                let bytes = file.read_to_vec();
                println!("Loaded File sleep, Size: {} Bytes", bytes.len());

                let elf = cashew_kernel::elf::parse(&bytes).expect("Failed To Parse ELF File");

                println!("ELF Header: {:#?}", elf.elf_header());
                println!("Entry Point: {:#010x}", elf.entry_point());

                println!("==== PROG. HEADERS ====");
                for header in elf.program_header_iter() {
                    println!("{:?} - {}B - Physical: 0x{:08x} - Virtual: 0x{:08x}",
                        header.ph_type(), 
                        header.filesz(), 
                        header.paddr(),
                        header.vaddr(),
                    );
                }

                println!("==== SECTIONS ====");
                for section in elf.section_header_iter() {
                    println!("{} - 0x{:08x} - {:?}", String::from_utf8_lossy(section.section_name()), section.addr(), section.flags());
                }



            },
            None => {println!("Failed To Open File...");},
        }


        ata::cache_stats();
        csh::main(Vec::new());
        cashew_kernel::shutdown();
    } else {
        kerr!("Failed To Find Framebuffer, Please File An Issue On Github.\n");
    }
    loop {
        cashew_kernel::arch::pause();
    }
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn userspace_prog_1() {
    asm!("nop");
}
