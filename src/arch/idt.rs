use crate::arch::pic;
use crate::{sprint, time};
use bit_field::BitField;
use lazy_static::lazy_static;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

use super::cmos::CMOS;
use super::pic::*;
use super::x64::structures::idt::InterruptDescriptorTable;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Interrupts {
    Timer = PIC1,
    Keyboard,

    Cmos = PIC1 + 8,

    Mouse = PIC1 + 12,

    AtaB0 = PIC1 + 14,
    AtaB1,
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
        idt.divide_error.set_handler_fn(divide_err);
        idt.general_protection_fault.set_handler_fn(gen_protection);

        idt[Interrupts::Timer.as_usize()].set_handler_fn(timer);
        idt[Interrupts::Keyboard.as_usize()].set_handler_fn(keyboard);

        idt[Interrupts::Cmos.as_usize()].set_handler_fn(cmos_nmi);

        idt[Interrupts::AtaB0.as_usize()].set_handler_fn(ata0);
        idt[Interrupts::AtaB1.as_usize()].set_handler_fn(ata1);

        idt
    };
}

pub fn initialize() {
    //super::gdt::init_gdt();
    IDT.load();
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    sprint!("#BP @ V${:08x}\n", frame.instruction_pointer.as_u64());
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _: u64) -> ! {
    panic!("#DF - RIP: V${:08x}\n", frame.instruction_pointer.as_u64())
}

extern "x86-interrupt" fn page_fault(_: InterruptStackFrame, ec: PageFaultErrorCode) {
    if (!ec.bits() & !PageFaultErrorCode::PROTECTION_VIOLATION.bits()) == 0 {
        crate::mem::map_virt(Cr2::read(), crate::mem::PTFlags::PRESENT | crate::mem::PTFlags::WRITABLE);
    }

    
}

extern "x86-interrupt" fn gen_protection(_: InterruptStackFrame, ec: u64) {
    sprint!("#GP - General Protection Fault {:#016X}...\n", ec);


    loop {}
}

extern "x86-interrupt" fn divide_err(sf: InterruptStackFrame) {
    sprint!(
        "#DE - ${:016x}\n", sf.instruction_pointer
    );
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

extern "x86-interrupt" fn ata0(_: InterruptStackFrame) {
    pic::notify_eoi(Interrupts::AtaB0.as_u8());
}

extern "x86-interrupt" fn ata1(_: InterruptStackFrame) {
    pic::notify_eoi(Interrupts::AtaB1.as_u8());
}


extern "x86-interrupt" fn cmos_nmi(_: InterruptStackFrame) {
    time::rtc_tick();
    CMOS::new().notify_end_of_interrupt();
    pic::notify_eoi(Interrupts::Cmos.as_u8());
}