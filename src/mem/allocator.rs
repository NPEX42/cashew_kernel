pub const HEAP_START: usize = 0x_000A_0000_0000;
pub const HEAP_SIZE: usize = 1024 * 1024 * 16; // 100 KiB

// in src/allocator.rs

#[global_allocator]
static LINKED_LIST_ALLOCATOR: LockedHeap = LockedHeap::empty();

use core::{alloc::Layout, panic, ptr::NonNull, sync::atomic::AtomicU64, cmp::Ordering, iter::Map};

use alloc::vec::Vec;
use bit_field::BitField;
use bootloader::{BootInfo, boot_info::{MemoryRegions, self, MemoryRegionKind}};
use conquer_once::spin::OnceCell;
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, Size4KiB, PhysFrame},
    VirtAddr, PhysAddr,
};

use x86_64::structures::paging::PageTableFlags as PTFlags;

use crate::{sprint, locked::Locked, println, klog};

use super::frames::BootInfoFrameAllocator;


pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {

    let mut memory_size = 0;
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
        "Mapping Page ${:x} - {:4}/{:4} - {:02.2}% - {:04} KB\n",
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


pub const PHYS_MEMORY_SIZE: usize = 256 << 24;
pub const PHYS_MEMORY_PAGES: usize = PHYS_MEMORY_SIZE / 4096;
pub const CHUNK_SIZE: usize = u128::BITS as usize;
pub static mut FRAME_BITMAP: [u128; PHYS_MEMORY_PAGES / CHUNK_SIZE] = [0; PHYS_MEMORY_PAGES / CHUNK_SIZE];
pub static mut ALLOC_COUNT: usize = 0;
pub static mut FREE_COUNT: usize = 0;
pub struct BitmapAllocator;


impl BitmapAllocator {

    pub fn free_count() -> usize {
        unsafe {FREE_COUNT}
    }

    pub fn used_count() -> usize {
        unsafe {ALLOC_COUNT}
    }

    fn alloc(frame_no: usize) {

        //klog!("Mapping Frame {:}\n", frame_no);

        let offset = frame_no / CHUNK_SIZE;
        let bit = frame_no % CHUNK_SIZE;
        
        unsafe {
            if offset >= FRAME_BITMAP.len() {return;}
            FRAME_BITMAP[offset].set_bit(bit, true);
        }
    }

    fn free(frame_no: usize) {
        let offset = frame_no / CHUNK_SIZE;
        let bit = frame_no % CHUNK_SIZE;

        unsafe {
            if offset >= FRAME_BITMAP.len() {return;}
            FRAME_BITMAP[offset].set_bit(bit, false);
        }
    }

    pub fn init(info: &BootInfo) {
        let regions = info.memory_regions.iter();
        let usable_regions = regions.filter(|r| r.kind != MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        let frames = frame_addresses.map(|addr| PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(addr)));

        let frame_count = frames.clone().count();

        unsafe { 
            ALLOC_COUNT = frame_count;
            FREE_COUNT = PHYS_MEMORY_PAGES - ALLOC_COUNT;
        }

        for frame in frames {
            for offset in 0..(frame.size() / 4096) {
                Self::alloc((frame.start_address().as_u64() / 4096) as usize + offset as usize);
            }
        }

    }

    pub fn next_free() -> Option<PhysFrame> {
        unsafe {
            for (index, block) in FRAME_BITMAP.iter().enumerate() {
                if *block == u128::MAX {continue;}

                for bit in 0..128 {
                    if !block.get_bit(bit) {
                        let frame_no = index * 128 + bit;
                        let start = frame_no * 4096;

                        Self::alloc(frame_no);

                        return Some(PhysFrame::containing_address(PhysAddr::new_truncate(start as u64)));
                    }
                }
            }

            None
        }
    }

    
 }


unsafe impl FrameAllocator<Size4KiB> for BitmapAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        Self::next_free()
    }
}


