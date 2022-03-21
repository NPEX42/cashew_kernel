#![no_std]
#![feature(once_cell)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(derive_default_enum)]
#![feature(const_btree_new)]
#![feature(const_fn_trait_bound)]
#![feature(int_log)]

use core::panic::PanicInfo;

pub mod api;

pub mod arch;
pub mod ata;
pub mod csh;
pub mod data;
pub mod device;
pub mod fonts;
pub mod fuse;
pub mod graphics_2d;
pub mod input;
pub mod locked;
pub mod logger;
pub mod mem;
pub mod pit;
pub mod serial;
pub mod terminal;
pub mod time;
pub mod vfs;
pub mod vga;

pub mod post;

pub mod wasi;

extern crate alloc;

pub use alloc::*;
use arch::cmos;
use bootloader::BootInfo;
use x86_64::VirtAddr;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sprint!("Panic: {}\n", info);
    kerr!("== Kernel Panic ==\n{}", info);
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[alloc_error_handler]
fn alloc_error(layout: alloc::alloc::Layout) -> ! {
    panic!(
        "Allocation Error: Unable To Allocate {} Bytes (Align: {})",
        layout.size(),
        layout.align()
    );
}

pub fn boot(info: &'static mut BootInfo) {
    if let Some(fb) = info.framebuffer.as_mut() {
        vga::initialize(fb.buffer_mut().as_mut_ptr(), fb.info());
        terminal::initialize();
        arch::initialize_interrupts();
        arch::enable_interrupts();
        pit::set_frequency(0, 60);

        input::init();
        let physical_memory_offset = info.physical_memory_offset.into_option().unwrap();
        let phys_mem_offset = VirtAddr::new(physical_memory_offset);
        mem::allocator::BitmapAllocator::init(info);
        mem::setup_from(info);
        mem::init(phys_mem_offset, &*info.memory_regions);

        cmos::CMOS::new().enable_periodic_interrupt();
        time::set_rate(3);

        println!(
            "Main Processor: '{:?}' - SSE {} - SSE2 {} - AVX {}",
            arch::cpu::vendor_info(),
            arch::cpu::supports_sse(),
            arch::cpu::supports_sse2(),
            arch::cpu::supports_avx(),
        );

        if let Some(cparams) = arch::cpu::cache_params() {
            for cache in cparams {
                let associativity = cache.associativity();
                let line_length = cache.coherency_line_size();
                let level = cache.level();
                println!("L{}-Cache: [{}:{}]", level, line_length, associativity);
            }
        } else {
            println!("No Cache Detected...");
        }

        post::self_test();
    }
}

pub fn shutdown() -> ! {
    arch::acpi::shutdown();

    loop {}
}
