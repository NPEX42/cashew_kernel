use core::{alloc::Layout, ptr::NonNull, sync::atomic::AtomicU64, ops::Add};


use bootloader::{boot_info::{MemoryRegion, MemoryRegions}, BootInfo};
use conquer_once::spin::OnceCell;
use x86_64::{
    instructions::interrupts::without_interrupts, structures::paging::{PageTable, Mapper, Size4KiB, PhysFrame, page::{self, PageRange}}, registers::control::Cr3
};

pub use x86_64::VirtAddr;
pub use x86_64::PhysAddr;
pub use x86_64::structures::paging::Page;
pub use x86_64::structures::paging::PageTableFlags as PTFlags;
pub use x86_64::structures::paging::{OffsetPageTable, Translate};

pub mod allocator;
pub mod frames;
pub mod mapper;
pub mod pagetable;

static mut PHYSICAL_OFFSET: Option<VirtAddr> = None;
pub static mut MEMORY_MAP: Option<&MemoryRegions> = None;
pub static MEMORY_SIZE: AtomicU64 = AtomicU64::new(0);

use crate::{locked::Locked, println, sprint, pit, csh::{ShellArgs, ExitCode}, mem::allocator::HEAP_SIZE};

use self::frames::BootInfoFrameAllocator;

static PAGE_TABLE: OnceCell<Locked<OffsetPageTable>> = OnceCell::uninit();

pub fn setup_from(info: &'static BootInfo) {
    unsafe {
        MEMORY_MAP.replace(&info.memory_regions);
        let mut memory_size = 0;
        for region in info.memory_regions.iter() {
            let start_addr = region.start;
            let end_addr = region.end;
            memory_size += end_addr - start_addr;
            println!("MEM [{:#016X}-{:#016X}] {:?} ({} KB)", start_addr, end_addr, region.kind, (end_addr - start_addr) / 1024);
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
    use x86_64::registers::control::Cr3;


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
            sprint!("L4 Entry {}: {:x} - {:?}\n", i, l4_entry.addr(), l4_entry.flags());

            let pt3 = pagetable_at_frame(l4_entry.frame().unwrap());
            for (i, l3_entry) in pt3.iter().enumerate() {
                if !l3_entry.is_unused() {
                    sprint!("-L3 Entry {}: {:x} - {:?}\n", i, l3_entry.addr(), l3_entry.flags());

                    if l3_entry.flags().contains(PTFlags::HUGE_PAGE) {continue;}

                    let pt2 = pagetable_at_frame(l3_entry.frame().unwrap());
                    for (i, l2_entry) in pt2.iter().enumerate() {
                        if !l2_entry.is_unused() {
                            sprint!("--L2 Entry {}: {:x} - {:?}\n", i, l2_entry.addr(), l2_entry.flags());
                            //pit::sleep(5);
                            if l2_entry.flags().contains(PTFlags::HUGE_PAGE) {continue;}

                            let pt1 = pagetable_at_frame(l2_entry.frame().unwrap());
                            for (i, l1_entry) in pt1.iter().enumerate() {
                                if !l1_entry.is_unused() {
                                    sprint!("---L1 Entry {}: {:x} --> {:x?}\n", i, l1_entry.addr(), phys_to_virt(l1_entry.addr()));
                                }
                            }
                        }
                    }

                }
            }

        }
    }

}


unsafe fn pagetable_at(addr: VirtAddr) -> PageTable {
    (*addr.as_mut_ptr::<PageTable>()).clone()


}

pub fn pagetable_at_frame(frame: PhysFrame) -> &'static PageTable {
    let phys = frame.start_address();
    let virt = phys.as_u64() + unsafe {PHYSICAL_OFFSET.unwrap().as_u64()};
    let ptr = VirtAddr::new(virt).as_mut_ptr();
    unsafe { &*ptr }
}


pub fn map_virt_to_phys(virt: VirtAddr, phys: PhysAddr, flags: PTFlags) {
    let mut mapper = PAGE_TABLE.get().unwrap().lock();
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&*MEMORY_MAP.unwrap()) };
    let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys);
    let page: Page<Size4KiB> = Page::containing_address(virt);
    unsafe { mapper.map_to(page, frame, flags, &mut frame_allocator).expect("Failed To Create Mapping").flush() };
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
            mapper.map_to(page, frame, flags, &mut frame_allocator).expect("Failed To Map Pages").flush();
        }
        phys_addr += 4096u64;
    }

}




pub fn csh_stats(_: ShellArgs) -> ExitCode {
    println!("=== MEM STATS ===");
    let free = free();
    let used = used();
    let total = HEAP_SIZE;

    let mut width = 0;
    width = width.max(free.log10());
    width = width.max(used.log10());
    width = width.max(total.log10());
    println!("Used:  {:0w$} Bytes", used, w=width as usize);
    println!("Free:  {:0w$} Bytes", free, w=width as usize);
    println!("Total: {:0w$} Bytes", total, w=width as usize);

    sprint!("Used:  {:0w$} Bytes\n", used, w=width as usize);
    sprint!("Free:  {:0w$} Bytes\n", free, w=width as usize);
    sprint!("Total: {:0w$} Bytes\n", total, w=width as usize);
    println!("=================");
    ExitCode::Ok
}