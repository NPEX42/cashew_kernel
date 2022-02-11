use lazy_static::lazy_static;
use x86_64::{structures::{gdt::*, tss::TaskStateSegment}, VirtAddr, instructions::{segmentation::*, tables::load_tss}};

use crate::{println, sprint};
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static mut STACK: [u8; 4096 * 5] = [0; 4096 * 5];
static mut USR_STACK: [u8; 4096 * 5] = [0; 4096 * 5];

lazy_static! {

    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + (4096 * 5) as u64;
            stack_end
        };
        tss
    };

    static ref GDT: (GlobalDescriptorTable, [SegmentSelector; 5]) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_data_flags = DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT | DescriptorFlags::WRITABLE;
        let code_sel = gdt.add_entry(Descriptor::kernel_code_segment()); // kernel code segment
        let data_sel = gdt.add_entry(Descriptor::UserSegment(kernel_data_flags.bits())); // kernel data segment
        let tss_sel = gdt.add_entry(Descriptor::tss_segment(&TSS)); // task state segment
        let user_data_sel = gdt.add_entry(Descriptor::user_data_segment()); // user data segment
        let user_code_sel = gdt.add_entry(Descriptor::user_code_segment()); // user code segment
        (gdt, [code_sel, data_sel, tss_sel, user_data_sel, user_code_sel])
    };
}

pub fn init_gdt() {
    sprint!("Loading GDT\n");
    GDT.0.load();
    let stack = unsafe { &STACK as *const _ };
    let user_stack = unsafe { &USR_STACK as *const _ };
    println!(
        " - Loaded GDT: {:p} TSS: {:p} Stack {:p} User stack: {:p} CS segment: {} TSS segment: {}",
        &GDT.0 as *const _, &*TSS as *const _, stack, user_stack, GDT.1[0].0, GDT.1[1].0
    );

    sprint!("Setting Segment Selectors\n");
    unsafe {
        CS::set_reg(GDT.1[0]);
        CS::set_reg(GDT.1[1]);
        load_tss(GDT.1[2]);
    }
}
