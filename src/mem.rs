use core::{alloc::Layout, ptr::NonNull};

use bootloader::boot_info::MemoryRegion;
use conquer_once::spin::OnceCell;
use x86_64::{
    instructions::interrupts::without_interrupts,
    structures::paging::{OffsetPageTable, Translate},
    PhysAddr, VirtAddr,
};

pub mod allocator;
pub mod frames;
pub mod mapper;
pub mod pagetable;

static mut PHYSICAL_OFFSET: Option<VirtAddr> = None;

use crate::locked::Locked;

use self::frames::BootInfoFrameAllocator;

static PAGE_TABLE: OnceCell<Locked<OffsetPageTable>> = OnceCell::uninit();

pub fn init(phys_offset: VirtAddr, regions: &'static [MemoryRegion]) {
    unsafe { PHYSICAL_OFFSET = Some(phys_offset) }

    unsafe {
        PAGE_TABLE.init_once(|| Locked::new(pagetable::init(phys_offset)));

        let mut mapper = PAGE_TABLE.get().unwrap().lock();
        let mut frame_allocator = BootInfoFrameAllocator::init(regions);
        allocator::init_heap(&mut *mapper, &mut frame_allocator).expect("Memory Init Failed");
    }
}

pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    if let Some(mapper) = PAGE_TABLE.get() {
        without_interrupts(|| {
            let mapper = mapper.lock();
            mapper.translate_addr(addr)
        })
    } else {
        None
    }
}

pub fn phys_to_virt(addr: PhysAddr) -> Option<VirtAddr> {
    if let Some(offset) = unsafe { PHYSICAL_OFFSET } {
        return Some(VirtAddr::new_truncate(addr.as_u64() - offset.as_u64()));
    } else {
        return None;
    }
}

pub fn malloc(size: usize, align: usize) -> NonNull<u8> {
    allocator::_malloc(Layout::from_size_align(size, align).expect("Alignment Error"))
}

pub fn used() -> usize {
    allocator::_used()
}

pub fn free() -> usize {
    allocator::_free()
}

pub fn dealloc(ptr: NonNull<u8>, size: usize, align: usize) {
    allocator::_dealloc(
        ptr,
        Layout::from_size_align(size, align).expect("Alignment Error"),
    );
}
