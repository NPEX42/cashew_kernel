use x86_64::{structures::paging::PageTable, VirtAddr};

use crate::fuse::Fuse;

static mut FUSE: Fuse = Fuse::new();

pub unsafe fn current_l4_table(offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    FUSE.test();

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

use x86_64::structures::paging::OffsetPageTable;

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = current_l4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}
