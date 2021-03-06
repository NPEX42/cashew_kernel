use crate::arch::x64::instructions::port;
use crate::device::BlockAddr;

use crate::pit::sleep;
use crate::println;
use crate::sprint;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::mem::size_of;

use alloc::string::String;
use bit_field::BitField;
use port::Port;
use port::PortReadOnly as PortR;
use port::PortWriteOnly as PortW;

pub const BLOCK_SIZE: usize = 512;

pub const CACHE_LINE_SIZE: u32 = 8;

static mut BLOCK_CACHE: BTreeMap<(u8, u8, u32), [u8; BLOCK_SIZE]> = BTreeMap::new();

static mut CACHE_MISSES: usize = 0;
static mut CACHE_HITS: usize = 0;
static mut TOTAL_OPS: usize = 0;

#[allow(deprecated)]
pub fn write_block(bus: u8, drive: u8, block: BlockAddr, data: &[u8]) -> EmptyResult {
    unsafe {
        let mut buf = [0; BLOCK_SIZE];
        buf.copy_from_slice(data);
        BLOCK_CACHE.insert((bus, drive, block), buf);
        write(bus, drive, block, data)?;
        Ok(())
    }
}

#[allow(deprecated)]
pub fn read_block(bus: u8, drive: u8, addr: u32) -> Result<[u8; BLOCK_SIZE], ()> {
    unsafe {
        TOTAL_OPS += 1;
        if BLOCK_CACHE.contains_key(&(bus, drive, addr)) {
            CACHE_HITS += 1;
            return Ok(*BLOCK_CACHE.get(&(bus, drive, addr)).unwrap());
        } else {
            let data = read(bus, drive, addr)?;
            BLOCK_CACHE.insert((bus, drive, addr), data);

            for i in 0..CACHE_LINE_SIZE {
                if let Ok(data_next) = read(bus, drive, addr + i) {
                    BLOCK_CACHE.insert((bus, drive, addr + i), data_next);
                }
            }

            CACHE_MISSES += 1;
            return Ok(data);
        }
    }
}

pub fn cache_stats() {
    println!("==== Cache Stats ====");
    println!("Misses: {:04}/{:04}", misses(), total_ops());
    println!("Hits:   {:04}/{:04}", hits(), total_ops());
    println!("Availability: {:02.3}%", availability() * 100.0);
    println!("=====================");
}

pub fn hits() -> usize {
    unsafe { CACHE_HITS }
}

pub fn misses() -> usize {
    unsafe { CACHE_MISSES }
}

pub fn total_ops() -> usize {
    unsafe { TOTAL_OPS }
}

pub fn availability() -> f64 {
    hits() as f64 / total_ops() as f64
}

pub type DiskResult<T> = Result<T, ()>;
pub type EmptyResult = Result<(), ()>;
pub type Sector = [u8; BLOCK_SIZE];

#[allow(dead_code)]
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Status {
    ERR = 0,  // Error
    IDX = 1,  // (obsolete)
    CORR = 2, // (obsolete)
    DRQ = 3,  // Data Request
    DSC = 4,  // (command dependant)
    DF = 5,   // (command dependant)
    DRDY = 6, // Device Ready
    BSY = 7,  // Busy
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    Indentify = 0xEC,
}

#[allow(unused)]
pub struct Registers {
    data: Port<u16>,
    error: PortR<u8>,
    features: PortW<u8>,
    sector_count: Port<u8>,

    lba_lo: Port<u8>,
    lba_mid: Port<u8>,
    lba_hi: Port<u8>,

    drive_sel: Port<u8>,

    command: PortW<u8>,
    status: PortR<u8>,

    // Control Registers
    alt_status: PortR<u8>,
    control: PortW<u8>,
    drive_addr: PortR<u8>,

    irq_num: u8,
}

impl Registers {
    pub fn new(io_base: u16, ctl_base: u16, irq: u8) -> Self {
        Self {
            data: Port::new(io_base + 0),
            error: PortR::new(io_base + 1),
            features: PortW::new(io_base + 1),
            sector_count: Port::new(io_base + 2),
            lba_lo: Port::new(io_base + 3),
            lba_mid: Port::new(io_base + 4),
            lba_hi: Port::new(io_base + 5),
            drive_sel: Port::new(io_base + 6),
            command: PortW::new(io_base + 7),
            status: PortR::new(io_base + 7),

            alt_status: PortR::new(ctl_base + 0),
            control: PortW::new(ctl_base + 0),
            drive_addr: PortR::new(ctl_base + 1),

            irq_num: irq,
        }
    }

    pub fn set_active_drive(&mut self, drive: u8) -> Result<(), ()> {
        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;

        unsafe {
            self.drive_sel.write(0xA0 | (drive << 4));
        }

        sleep(1);

        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, false)?;

        Ok(())
    }

    pub fn poll(&mut self, bit: Status, val: bool) -> EmptyResult {
        let start = crate::pit::uptime();
        while unsafe { self.status.read().get_bit(bit as usize) != val } {
            if crate::pit::uptime() - start > (crate::pit::polling_rate() * 1) {
                self.debug();
                return Err(());
            }
        }

        Ok(())
    }

    fn read_data(&mut self) -> u16 {
        unsafe { self.data.read() }
    }

    fn write_data(&mut self, data: u16) {
        unsafe { self.data.write(data) }
    }

    fn is_error(&mut self) -> bool {
        self.status().get_bit(Status::ERR as usize)
    }

    pub fn clear_interrupt(&mut self) {
        unsafe {
            self.status.read();
        }
    }

    pub fn status(&mut self) -> u8 {
        unsafe { self.alt_status.read() }
    }

    fn command(&mut self, cmd: Command) -> Result<(), ()> {
        unsafe { self.command.write(cmd as u8) };
        sleep(1);
        self.status();
        self.clear_interrupt();

        if self.status() == 0 {
            // Drive Nonexistent
            return Err(());
        }

        if self.is_error() {
            // Command Failed
            return Err(());
        }

        self.poll(Status::BSY, false)?;
        self.poll(Status::DRQ, true)?;
        Ok(())
    }

    fn setup_pio(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        self.set_active_drive(drive)?;
        self.write_command_params(drive, block)?;
        Ok(())
    }

    fn write_command_params(&mut self, drive: u8, block: u32) -> Result<(), ()> {
        let lba = true;
        let mut bytes = block.to_le_bytes();
        bytes[3].set_bit(4, drive > 0);
        bytes[3].set_bit(5, true);
        bytes[3].set_bit(6, lba);
        bytes[3].set_bit(7, true);
        unsafe {
            self.sector_count.write(1);
            self.lba_lo.write(bytes[0]);
            self.lba_mid.write(bytes[1]);
            self.lba_hi.write(bytes[2]);
            self.drive_sel.write(bytes[3]);
        }
        Ok(())
    }

    fn debug(&mut self) {
        sprint!("Status: 0b{:08b} - <BSY|DRDY|#|#|DRQ|#|#|ERR>\n", unsafe {
            self.status.read()
        });
        sprint!("Error:  0b{:08b} - <#|#|#|#|#|ABRT|#|#>\n", unsafe {
            self.error.read()
        })
    }

    pub fn read_block(&mut self, drive: u8, block: u32) -> Result<Sector, ()> {
        self.setup_pio(drive, block)?;
        self.command(Command::Read)?;
        let mut buffer: Sector = [0; BLOCK_SIZE];

        for chunk in buffer.chunks_mut(size_of::<u16>()) {
            let data = self.read_data().to_le_bytes();
            chunk.copy_from_slice(&data);
        }

        if self.is_error() {
            return Err(());
        }

        Ok(buffer)
    }

    fn write_block(&mut self, drive: u8, block: u32, buf: &[u8]) -> Result<(), ()> {
        debug_assert!(buf.len() == BLOCK_SIZE);
        self.setup_pio(drive, block)?;
        self.command(Command::Write)?;
        for chunk in buf.chunks(2) {
            let data = u16::from_le_bytes(chunk.try_into().unwrap());
            self.write_data(data);
        }
        if self.is_error() {
            self.debug();
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn indentify(&mut self, drive: u8) -> Result<DiskInfo, ()> {
        self.set_active_drive(drive)?;
        self.write_command_params(drive, 0)?;

        self.command(Command::Indentify)?;

        let mut info = DiskInfo::empty();

        let data = [(); 256].map(|_| self.read_data());
        let buf = data.map(u16::to_be_bytes).concat();
        let serial: String = String::from_utf8_lossy(&buf[20..40]).trim().into();
        let model: String = String::from_utf8_lossy(&buf[54..94]).trim().into();
        let blocks = u32::from_be_bytes(buf[120..124].try_into().unwrap()).rotate_left(16);

        info.model = model;
        info.serial = serial;
        info.sectors = blocks as usize;

        Ok(info)
    }
}

pub struct DiskInfo {
    pub serial: String,
    pub model: String,
    pub sectors: usize,
}

impl DiskInfo {
    pub fn empty() -> Self {
        Self {
            model: String::new(),
            sectors: 0,
            serial: String::new(),
        }
    }
}

static mut BUSES: Option<Vec<Bus>> = None;

pub fn init() {
    let mut buses = Vec::new();

    buses.push(Bus::bus_0());
    buses.push(Bus::bus_1());

    unsafe {BUSES = Some(buses);}
}

pub fn bus<'a>(index: u8) -> Option<&'a Bus> {
    if index > 1 { return None; }

    unsafe {
        if let Some(buses) = &BUSES {
            buses.get(index as usize)
        } else {
            return None;
        }
    }
}

#[deprecated]
/// MARKED FOR INTERNAL USE ONLY
pub fn read(bus: u8, drive: u8, block: u32) -> Result<Sector, ()> {
    let mut bus = get_register(bus);
    bus.read_block(drive, block)
}

#[deprecated]
/// MARKED FOR INTERNAL USE ONLY
pub fn write(bus: u8, drive: u8, block: u32, data: &[u8]) -> EmptyResult {
    let mut bus = get_register(bus);
    bus.write_block(drive, block, data)
}

pub fn get_sector_count(bus: u8, drive: u8) -> Result<usize, ()> {
    let mut bus = get_register(bus);
    let info = bus.indentify(drive)?;
    Ok(info.sectors)
}

pub fn info(bus: u8, drive: u8) -> Result<DiskInfo, ()> {
    let mut bus = get_register(bus);
    bus.indentify(drive)
}

/// bus #0 => ($1F0, $3F6, 14)
///
/// bus #1 => ($170, $376, 15)
fn get_register(bus: u8) -> Registers {
    let bus_io = if bus == 0 { 0x1F0 } else { 0x170 };
    let bus_ctl = if bus == 0 { 0x3F6 } else { 0x376 };
    let bus_irq = if bus == 0 { 14 } else { 15 };
    Registers::new(bus_io, bus_ctl, bus_irq)
}


pub struct Bus {
    registers: Registers,
    active_drive: DriveIndex,
}


/// (I/O Base, Control Base, IRQ line)
pub type BusValues = (u16, u16, u8);

pub const BUS_0: BusValues = (0x1F0, 0x3F6, 14);
pub const BUS_1: BusValues = (0x170, 0x376, 15);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum DriveIndex {
    Primary = 0,
    Secondary = 1,
}

impl Bus {

    pub fn bus_0() -> Self {
        Self::new(BUS_0)
    }
    pub fn bus_1() -> Self {
        Self::new(BUS_1)
    }

    pub fn new(values: BusValues) -> Self {
        Self {
            registers: Registers::new(values.0, values.1, values.2),
            active_drive: DriveIndex::Primary
        }
    }

    pub fn set_active_drive(&mut self, drive: DriveIndex) {
        self.active_drive = drive;
    }

    pub fn active_drive(&self) -> DriveIndex {
        self.active_drive
    }

    pub fn write(&mut self, addr: BlockAddr, data: &[u8]) -> DiskResult<()> {
        self.registers.write_block(self.active_drive as u8,addr, data)
    }

    pub fn read(&mut self, addr: BlockAddr) -> DiskResult<Sector> {
        self.registers.read_block(self.active_drive as u8, addr)
    }
}