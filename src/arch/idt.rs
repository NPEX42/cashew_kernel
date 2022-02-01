use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use crate::arch::pic;
use crate::{sprint};

use super::x64::structures::idt::InterruptDescriptorTable;
use super::pic::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Interrupts {
    Timer = PIC1,
    Keyboard,
    Mouse = PIC1 + 12
}

impl Interrupts {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    } 

    pub fn as_usize(&self) -> usize {
        *self as usize
    } 
}


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint); 
        idt.double_fault.set_handler_fn(double_fault);
        idt.page_fault.set_handler_fn(page_fault);


        idt[Interrupts::Timer.as_usize()].set_handler_fn(timer);
        idt[Interrupts::Keyboard.as_usize()].set_handler_fn(keyboard);
        idt[Interrupts::Mouse.as_usize()].set_handler_fn(mouse);

        idt
    };
}

pub fn initialize() {
    IDT.load();
}


extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    sprint!("Breakpoint @ V${:08x}\n", frame.instruction_pointer.as_u64());
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _: u64) -> ! {
    panic!("#DF - RIP: V${:08x}\n", frame.instruction_pointer.as_u64())
}

extern "x86-interrupt" fn page_fault(_: InterruptStackFrame, ec: PageFaultErrorCode) {
    sprint!("#PF - {:?} - CR2: {:x?}", ec, super::x64::registers::control::Cr2);
}

extern "x86-interrupt" fn timer(_: InterruptStackFrame) {
    //crate::sprint!("Tick!\n");
    crate::pit::update_timers();
    crate::graphics_2d::vblank();
    pic::notify_eoi(Interrupts::Timer.as_u8())
}

extern "x86-interrupt" fn keyboard(_: InterruptStackFrame) {
    crate::input::keyboard::keypress();
    pic::notify_eoi(Interrupts::Keyboard.as_u8());
}

extern "x86-interrupt" fn mouse(_: InterruptStackFrame) {
    crate::input::mouse::update();
    sprint!("Mouse Moved!\n");
    pic::notify_eoi(Interrupts::Mouse.as_u8());
}