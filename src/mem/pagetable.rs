use core::ops::{Index, IndexMut};

use x86_64::{structures::paging::{self, PageTable, page_table::PageTableEntry}, VirtAddr};

use crate::fuse::Fuse;

static mut FUSE: Fuse = Fuse::new();

pub unsafe fn current_l4_table(offset: VirtAddr) -> &'static mut paging::PageTable {
    use x86_64::registers::control::Cr3;

    FUSE.test();

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = offset + phys.as_u64();
    let page_table_ptr: *mut paging::PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

use x86_64::structures::paging::OffsetPageTable;

use super::{PHYSICAL_OFFSET, pagetable_at_frame};

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


pub struct PageTableWrapper(paging::PageTable);

impl PageTableWrapper {
    pub unsafe fn from_active() -> Self {
        Self(current_l4_table(PHYSICAL_OFFSET.unwrap()).clone())
    }

    pub fn from(table: PageTable) -> Self {
        Self(table)
    }

    pub fn next_pt(&self, index: usize) -> Option<Self> {
        if self.0[index].is_unused() {return None;}
        unsafe {
            if let Ok(frame) = self[index].frame() {
                return Some(Self::from(pagetable_at_frame(frame).clone()));
            } else {
                return None;
            }
        }
    }
    
}

impl Index<usize> for PageTableWrapper {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
} 

impl IndexMut<usize> for PageTableWrapper {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
} 