pub const HEAP_START: usize = 0x_000A_0000_0000;
pub const HEAP_SIZE: usize = 1024 * 1024; // 100 KiB

// in src/allocator.rs

#[global_allocator]
static LINKED_LIST_ALLOCATOR: LockedHeap = LockedHeap::empty();

use core::{alloc::Layout, panic, ptr::NonNull};

use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, Size4KiB},
    VirtAddr,
};

use x86_64::structures::paging::PageTableFlags as PTFlags;

use crate::sprint;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    let mut count = 0;
    let total = HEAP_SIZE / 4096;
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PTFlags::PRESENT | PTFlags::WRITABLE | PTFlags::BIT_10;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
        if count % (total / 100) == 0 {
            sprint!(
                "Mapping Page ${:x} - {:4}/{:4} - {:02.2}% - {:04} KB\r",
                page.start_address(),
                count,
                total,
                (count as f32 / total as f32) * 100.0,
                (count * 4096) / 1024
            );
        }
        count += 1;
    }

    sprint!(
        "Mapping Page ${:x} - {:4}/{:4} - {:02.2}% - {:04} KB\r",
        (HEAP_SIZE + HEAP_START),
        count,
        total,
        (count as f32 / total as f32) * 100.0,
        (count * 4096) / 1024
    );

    unsafe {
        LINKED_LIST_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub(super) fn _malloc(layout: Layout) -> NonNull<u8> {
    if let Ok(ptr) = LINKED_LIST_ALLOCATOR.lock().allocate_first_fit(layout) {
        ptr
    } else {
        panic!(
            "Allocation Error: Unable To Allocate {} Bytes (Align: {})",
            layout.size(),
            layout.align()
        );
    }
}

pub(super) fn _used() -> usize {
    LINKED_LIST_ALLOCATOR.lock().used()
}

pub(super) fn _free() -> usize {
    LINKED_LIST_ALLOCATOR.lock().free()
}

pub(super) fn _dealloc(ptr: NonNull<u8>, layout: Layout) {
    unsafe {
        LINKED_LIST_ALLOCATOR.lock().deallocate(ptr, layout);
    }
}
