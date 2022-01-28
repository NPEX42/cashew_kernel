#![no_std]
#![feature(once_cell)]
#![feature(abi_x86_interrupt)]
//#![feature(generic_const_exprs)]

pub mod vga;
pub mod logger;
pub mod locked;
pub mod serial;
pub mod fonts;
pub mod terminal;
pub mod colors;
pub mod pit;
pub mod graphics_2d;
pub mod input;

pub mod arch;