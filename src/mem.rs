use core::{alloc::Layout, ptr::NonNull, sync::atomic::AtomicU64};

use alloc::string::ToString;
use bootloader::{
    boot_info::{MemoryRegion, MemoryRegions},
    BootInfo,
};
use conquer_once::spin::OnceCell;
use x86_64::{
    instructions::interrupts::without_interrupts,
    registers::control::Cr3,
    structures::paging::{Mapper, PageTable, PhysFrame, Size4KiB},
};

pub use x86_64::structures::paging::Page;
pub use x86_64::structures::paging::PageTableFlags as PTFlags;
pub use x86_64::structures::paging::{OffsetPageTable, Translate};
pub use x86_64::PhysAddr;
pub use x86_64::VirtAddr;

pub mod allocator;
pub mod frames;
pub mod mapper;
pub mod pagetable;

static mut PHYSICAL_OFFSET: Option<VirtAddr> = None;
pub static mut MEMORY_MAP: Option<&MemoryRegions> = None;
pub static MEMORY_SIZE: AtomicU64 = AtomicU64::new(0);

use crate::{
    csh::{ExitCode, ShellArgs},
    klog,
    locked::Locked,
    mem::allocator::HEAP_SIZE,
    pit, println, sprint,
};

use self::{allocator::BitmapAllocator, frames::BootInfoFrameAllocator};

static PAGE_TABLE: OnceCell<Locked<OffsetPageTable>> = OnceCell::uninit();

pub fn setup_from(info: &'static BootInfo) {
    unsafe {
        MEMORY_MAP.replace(&info.memory_regions);
        let mut memory_size = 0;
        for region in info.memory_regions.iter() {
            let start_addr = region.start;
            let end_addr = region.end;
            memory_size += end_addr - start_addr;
            println!(
                "MEM [{:#016X}-{:#016X}] {:?} ({} KB)",
                start_addr,
                end_addr,
                region.kind,
                (end_addr - start_addr) / 1024
            );
        }
        MEMORY_SIZE.store(memory_size, core::sync::atomic::Ordering::Relaxed);
    }
}

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
        return Some(VirtAddr::new_truncate(addr.as_u64() + offset.as_u64()));
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

pub unsafe fn clone_pagetable() -> PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = PHYSICAL_OFFSET.unwrap() + phys.as_u64();
    let page_table_clone: PageTable = (*virt.as_mut_ptr::<PageTable>()).clone();

    page_table_clone
}

pub fn debug_print_pagetables() {
    sprint!("=== Active Page Tables ===\n");

    sprint!("Cr3: {:?}\n", Cr3::read());
    let pt4 = pagetable_at_frame(Cr3::read().0);
    for (i, l4_entry) in pt4.iter().enumerate() {
        if !l4_entry.is_unused() {
            sprint!(
                "L4 Entry {}: {:x} - {:?}\n",
                i,
                l4_entry.addr(),
                l4_entry.flags()
            );

            let pt3 = pagetable_at_frame(l4_entry.frame().unwrap());
            for (i, l3_entry) in pt3.iter().enumerate() {
                if !l3_entry.is_unused() {
                    sprint!(
                        "-L3 Entry {}: {:x} - {:?}\n",
                        i,
                        l3_entry.addr(),
                        l3_entry.flags()
                    );

                    if l3_entry.flags().contains(PTFlags::HUGE_PAGE) {
                        continue;
                    }

                    let pt2 = pagetable_at_frame(l3_entry.frame().unwrap());
                    for (i, l2_entry) in pt2.iter().enumerate() {
                        if !l2_entry.is_unused() {
                            sprint!(
                                "--L2 Entry {}: {:x} - {:?}\n",
                                i,
                                l2_entry.addr(),
                                l2_entry.flags()
                            );

                            if l2_entry.flags().contains(PTFlags::HUGE_PAGE) {
                                continue;
                            }

                            let pt1 = pagetable_at_frame(l2_entry.frame().unwrap());
                            for (i, l1_entry) in pt1.iter().enumerate() {
                                if !l1_entry.is_unused() {
                                    sprint!(
                                        "---L1 Entry {}: {:x?} --> {:x}\n",
                                        i,
                                        phys_to_virt(l1_entry.addr()),
                                        (l1_entry.addr())
                                    );
                                    pit::sleep(5);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn debug_simple_mapping_exc(start: VirtAddr, end: VirtAddr) {
    for page in Page::<Size4KiB>::range(
        Page::containing_address(start),
        Page::containing_address(end),
    ) {
        if is_mapped(page.start_address()) {
            let v = page.start_address();
            let phys = virt_to_phys(v).unwrap();

            let p4 = usize::from(page.p4_index());
            let p3 = usize::from(page.p3_index());
            let p2 = usize::from(page.p2_index());
            let p1 = usize::from(page.p1_index());

            sprint!(
                "Map[{:03}][{:03}][{:03}][{:03}] = {:#x}\n",
                p4,
                p3,
                p2,
                p1,
                phys
            )
        }
    }
}

pub fn pagetable_at_frame(frame: PhysFrame) -> &'static PageTable {
    let phys = frame.start_address();
    let virt = phys.as_u64() + unsafe { PHYSICAL_OFFSET.unwrap().as_u64() };
    let ptr = VirtAddr::new(virt).as_mut_ptr();
    unsafe { &*ptr }
}

pub fn map_virt_to_phys(virt: VirtAddr, phys: PhysAddr, flags: PTFlags) {
    let mut mapper = PAGE_TABLE.get().unwrap().lock();
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&*MEMORY_MAP.unwrap()) };
    let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys);
    let page: Page<Size4KiB> = Page::containing_address(virt);
    unsafe {
        match mapper.map_to(page, frame, flags, &mut frame_allocator) {
            Ok(flsh) => flsh.flush(),
            Err(ec) => match ec {
                x86_64::structures::paging::mapper::MapToError::FrameAllocationFailed => {
                    panic!("Failed To Allocate Frame!")
                }
                x86_64::structures::paging::mapper::MapToError::ParentEntryHugePage => {
                    klog!("Huge Page Already Mapped...\n")
                }
                x86_64::structures::paging::mapper::MapToError::PageAlreadyMapped(frame) => {
                    panic!("Frame {:?} Is Already Mapped!", frame)
                }
            },
        }
    }
}

pub fn map_virt(virt: VirtAddr, flags: PTFlags) {
    let _frame_allocator = unsafe { BootInfoFrameAllocator::init(&*MEMORY_MAP.unwrap()) };
    if let Some(frame) = BitmapAllocator::next_free() {
        klog!(
            "Mapping ${:x} --> ${:x}\n",
            virt,
            frame.start_address() + (virt.as_u64() & 0xFFF)
        );
        map_virt_to_phys(virt, frame.start_address(), flags);
    } else {
        panic!("Out Of Physical Memory");
    }
}

pub fn map_contiguous(size: usize, virt: VirtAddr, phys: PhysAddr, flags: PTFlags) {
    let start_page: Page<Size4KiB> = Page::containing_address(virt);
    let end = VirtAddr::new(virt.as_u64() + size as u64);

    let end_page: Page<Size4KiB> = Page::containing_address(end);

    let pages = Page::range(start_page, end_page);
    let mut mapper = PAGE_TABLE.get().unwrap().lock();
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&*MEMORY_MAP.unwrap()) };

    let mut phys_addr = phys;
    for page in pages {
        let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys_addr);
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut frame_allocator)
                .expect("Failed To Map Pages")
                .flush();
        }
        phys_addr += 4096u64;
    }
}

pub fn is_mapped(virt: VirtAddr) -> bool {
    virt_to_phys(virt).is_some()
}

pub fn csh_stats(_: ShellArgs) -> ExitCode {
    println!("=== MEM STATS ===");
    let free = free();
    let used = used();
    let total = HEAP_SIZE;

    let mut width = 0;

    width = width.max(free.to_string().chars().count());
    width = width.max(used.to_string().chars().count());
    width = width.max(total.to_string().chars().count());

    println!("Used:  {:0>w$} Bytes", used, w = width as usize);
    println!("Free:  {:0>w$} Bytes", free, w = width as usize);
    println!("Total: {:0>w$} Bytes", total, w = width as usize);

    sprint!("Used:  {:0>w$} Bytes\n", used, w = width as usize);
    sprint!("Free:  {:0>w$} Bytes\n", free, w = width as usize);
    sprint!("Total: {:0>w$} Bytes\n", total, w = width as usize);
    println!("=================");
    ExitCode::Ok
}

// pub fn identity_map() -> PageTable {
//     let mut pagetable_4 = PageTable::new();
//     for entry in pagetable_4.iter_mut() {
//         let mut pagetable_3 = PageTable::new();
//         entry.set_addr(, PTFlags::PRESENT | PTFlags::WRITABLE);
//     }

//     pagetable_4
// }

pub unsafe fn read_mmio_u8(addr: VirtAddr) -> u8 {
    let ptr: *const u8 = addr.as_ptr();
    core::ptr::read_volatile(ptr)
}

pub unsafe fn write_mmio_u8(addr: VirtAddr, value: u8) {
    let ptr: *mut u8 = addr.as_mut_ptr();
    core::ptr::write_volatile(ptr, value)
}
