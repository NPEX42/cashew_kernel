use core::{arch::asm, fmt::Display, ops::Range};

use bit_field::BitField;
use x86_64::{registers::control::Cr3, structures::paging::PhysFrame, PhysAddr, VirtAddr};

use crate::{klog, mem};
#[derive(Clone, Copy, Debug, Default)]
pub struct PageTableEntry(u64);

#[derive(Clone, Copy, Debug)]
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

pub const PTF_PRESENT_BIT: usize = 0;
pub const PTF_WRITABLE_BIT: usize = 1;
pub const PTF_USER_BIT: usize = 2;
pub const PTF_WRITE_THROUGH_BIT: usize = 3;
pub const PTF_NO_CACHE_BIT: usize = 4;
pub const PTF_ACCESSED_BIT: usize = 5;
pub const PTF_DIRTY_BIT: usize = 6;
pub const PTF_HUGE_PAGE_BIT: usize = 7;
pub const PTF_GLOBAL_BIT: usize = 8;
pub const PTF_ADDRESS_BITS: Range<usize> = 12..52;

impl PageTableEntry {
    pub fn empty() -> Self {
        Self(0)
    }

    pub unsafe fn from(value: u64) -> Self {
        Self(value)
    }

    pub fn address(&self) -> u64 {
        return self.0.get_bits(PTF_ADDRESS_BITS) << 12;
    }

    pub fn flags(&self) -> u8 {
        return (self.0 & 0xFF) as u8;
    }

    pub fn set_present_bit(&mut self, state: bool) {
        self.set_bit(PTF_PRESENT_BIT, state);
    }

    pub fn phys_address(&self) -> PhysAddr {
        return PhysAddr::new_truncate(self.address());
    }

    pub fn set_bit(&mut self, bit: usize, value: bool) {
        self.0.set_bit(bit, value);
    }

    pub fn get_bit(&self, bit: usize) -> bool {
        let mask = 1 << bit;
        let result = self.flags() & mask;
        klog!("{:08b} & {:08b} = {}\n", self.flags(), mask, result);

        result != 0
    }

    pub fn is_present(&self) -> bool {
        let present = self.get_bit(PTF_PRESENT_BIT);
        //klog!("Entry is Present? {}.\n", present);
        present
    }

    pub fn to_page(&self) -> Option<Page> {
        if self.is_free() {
            return None;
        };

        Some(Page::from(
            VirtAddr::new(self.address()).align_down(4096 as u64),
        ))
    }

    pub fn to_page_table(&self) -> Option<PageTable> {
        if self.is_free() {
            return None;
        };

        unsafe {
            Some(PageTable::clone_from_phys_frame(
                PhysFrame::containing_address(self.phys_address()),
            ))
        }
    }

    pub fn is_free(&self) -> bool {
        !self.is_present()
    }
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry::empty(); 512],
        }
    }

    pub fn get_table(&self, index: usize) -> Option<Self> {
        unsafe {
            if self.entries[index].is_present() {
                Some(Self::clone_from_phys_frame(PhysFrame::containing_address(
                    self.entries[index].phys_address(),
                )))
            } else {
                None
            }
        }
    }

    pub unsafe fn clone_from_phys_frame(frame: PhysFrame) -> Self {
        let virt = mem::phys_to_virt(frame.start_address()).unwrap();
        let virt_ptr = virt.as_ptr() as *const PageTable;

        return *virt_ptr;
    }

    pub fn frame_addr(&self) -> Option<PhysAddr> {
        let virt = VirtAddr::from_ptr(self);
        let phys = mem::virt_to_phys(virt);

        phys
    }

    pub unsafe fn enable(&self) {
        let phys_addr = self.frame_addr().unwrap();
        klog!("Enabled PageTable At {:?}\n", self.frame_addr());
        asm!("mov {0}, cr3", in(reg) phys_addr.as_u64());
    }

    pub unsafe fn clone_from_cr3() -> Self {
        let pagetable = Self::clone_from_phys_frame(Cr3::read().0);
        klog!("Cloned PageTable At {:?}\n", pagetable.frame_addr());
        pagetable
    }

    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        self.entries.iter_mut()
    }
}

impl Display for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PageTableEntry({:08b}[HDACTUWP], ${:010x})",
            self.flags(),
            self.address()
        )
    }
}

pub struct Page {
    start: VirtAddr,
}

impl Page {
    /// ## Panics
    /// - Panics If The Address is not aligned on a 4096 Boundary (One Page).
    pub fn from(addr: VirtAddr) -> Page {
        assert!(addr.is_aligned(4096u64));
        Page { start: addr }
    }

    pub fn containing_address(addr: VirtAddr) -> Page {
        Self::from(addr.align_down(4096u64))
    }

    pub fn start(&self) -> VirtAddr {
        self.start
    }
    pub fn end(&self) -> VirtAddr {
        VirtAddr::new_truncate(self.start().as_u64() + 4096)
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.start().as_mut_ptr()
    }
}
