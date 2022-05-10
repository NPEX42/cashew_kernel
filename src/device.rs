use core::{ops::Range, fmt::{Write, Display}};

use alloc::{string::String, vec::Vec};

use crate::{
    ata::{self, Sector, BLOCK_SIZE},
    csh::{ErrorCode, ExitCode, ShellArgs},
    println, sprint,
    vfs::block::Block, serial, terminal, input, locked::{Locked, SharedChannel},
};

const MEM_DISK_SIZE: usize = (4 << 20) / BLOCK_SIZE;

pub type BlockAddr = u32;

static mut MOUNT: Option<Device> = None;

pub fn mount_main(args: ShellArgs) -> ExitCode {
    if args.len() < 2 {
        println!("Usage: {} [hda|hdb|hdc|hdd|mem]", args[0]);
        return ExitCode::Error(ErrorCode::Usage);
    }

    if let Some(dev) = Device::from_str(&args[1]) {
        if let Ok(info) = dev.clone().info() {
            println!("Mounted {}", info);
            mount(dev);
        } else {
            println!("Failed To Mount Device...");
            return ExitCode::Error(ErrorCode::FatalError(1));
        }
        
    } else {
        println!("Invalid Device: '{}'", args[1]);
        println!("Usage: {} [hda|hdb|hdc|hdd|mem]", args[0]);
        return ExitCode::Error(ErrorCode::Usage);
    }

    ExitCode::Ok
}

/// Returns The Size In Blocks of the currently mounted BlockDevice.
/// Returns Zero if no device is mounted.
pub fn blk_dev_size() -> usize {
    if is_mounted() {
        return info().expect("Failed To Read Info").blocks;
    } else {
        0
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub blocks: usize,
    pub name: String,
}

pub enum DeviceError {
    BufferTooSmall,
}

impl DeviceInfo {
    pub fn generic_device(size: usize) -> Self {
        Self {
            blocks: size,
            name: String::from("GENERIC DEVICE"),
        }
    }
}

pub trait BlockDeviceIO {
    fn read(&self, block: BlockAddr) -> Result<[u8; ata::BLOCK_SIZE], ()>;
    fn write(&mut self, block: BlockAddr, data: &[u8]) -> Result<(), ()>;

    fn read_range(
        &self,
        bounds: Range<BlockAddr>,
        buffer: &mut [[u8; ata::BLOCK_SIZE]],
    ) -> Result<(), ()> {
        if buffer.len() < bounds.len() {
            return Err(());
        }

        for (index, addr) in bounds.enumerate() {
            buffer[index] = self.read(addr)?;
        }

        Ok(())
    }

    fn write_range(
        &mut self,
        bounds: Range<BlockAddr>,
        buffer: &[[u8; ata::BLOCK_SIZE]],
    ) -> Result<(), ()> {
        if buffer.len() < bounds.len() {
            return Err(());
        }

        for (index, addr) in bounds.enumerate() {
            self.write(addr, &buffer[index])?;
        }

        Ok(())
    }

    fn block_count(&self) -> Result<usize, ()>;

    fn info(&self) -> Result<DeviceInfo, ()> {
        Ok(DeviceInfo::generic_device(self.block_count()?))
    }

    fn exists(&self) -> bool {
        self.info().is_ok()
    }
}

#[derive(Debug, Clone)]
pub enum Device {
    Ata(u8, u8),
    Mem(Vec<Sector>),
}

impl Device {
    pub fn hda() -> Self {
        Self::Ata(0, 0)
    }
    pub fn hdb() -> Self {
        Self::Ata(0, 1)
    }
    pub fn hdc() -> Self {
        Self::Ata(1, 0)
    }
    pub fn hdd() -> Self {
        Self::Ata(1, 1)
    }

    pub fn mem() -> Self {
        let mut disk = Vec::new();
        for _ in 0..MEM_DISK_SIZE { disk.push([0; BLOCK_SIZE]) }
        Self::Mem(disk)
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hda" => Some(Self::hda()),
            "hdb" => Some(Self::hdb()),
            "hdd" => Some(Self::hdc()),
            "hdc" => Some(Self::hdd()),
            "mem" => Some(Self::mem()),
            _ => None,
        }
    }
}

impl BlockDeviceIO for Device {
    fn block_count(&self) -> Result<usize, ()> {
        match self {
            Device::Ata(bus, drive) => ata::get_sector_count(*bus, *drive),
            Device::Mem(blocks) => Ok(blocks.len()),
        }
    }

    fn read(&self, block: BlockAddr) -> Result<[u8; ata::BLOCK_SIZE], ()> {
        match self {
            Device::Ata(bus, drive) => ata::read_block(*bus, *drive, block),
            Device::Mem(blocks) => {
                if let Some(block) = blocks.get(block as usize) {
                    return Ok(*block);
                } else {
                    return Err(());
                }
            }
        }
    }

    fn write(&mut self, block: BlockAddr, data: &[u8]) -> Result<(), ()> {
        match self {
            Device::Ata(bus, drive) => ata::write_block(*bus, *drive, block, data),
            Device::Mem(blocks) => {
                if blocks.len() > block as usize {
                    blocks[block as usize] = data.try_into().expect("msg");
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    fn info(&self) -> Result<DeviceInfo, ()> {
        match self {
            Device::Ata(bus, drive) => {
                let info = ata::info(*bus, *drive)?;

                Ok(DeviceInfo {
                    blocks: info.sectors,
                    name: info.model + ":" + &info.serial,
                })
            }

            Device::Mem(blocks) => {
                Ok(DeviceInfo {
                    blocks: blocks.len(),
                    name: "MEMORY".into(),
                })
            },
        }
    }
}

pub fn mount(dev: Device) {
    if !dev.exists() {
        sprint!("[{}]: Cannot Mount Device\n", module_path!());
        return;
    }
    unsafe {
        MOUNT = Some(dev);
    }
}

pub fn read(block: BlockAddr) -> Result<[u8; 512], ()> {
    if let Some(dev) = unsafe { &mut MOUNT } {
        dev.read(block)
    } else {
        Err(())
    }
}

pub fn read_block(block: BlockAddr) -> Result<Block, ()> {
    if let Some(dev) = unsafe { &mut MOUNT } {
        let data = dev.read(block)?;
        Ok(Block::from(block, data))
    } else {
        Err(())
    }
}

pub fn write(block: BlockAddr, data: &[u8]) -> Result<(), ()> {
    if let Some(dev) = unsafe { &mut MOUNT } {
        dev.write(block, data)
    } else {
        Err(())
    }
}

pub fn write_block(addr: BlockAddr, block: Block) -> Result<(), ()> {
    if let Some(dev) = unsafe { &mut MOUNT } {
        dev.write(addr, block.data())
    } else {
        Err(())
    }
}

pub fn info() -> Result<DeviceInfo, ()> {
    if let Some(dev) = unsafe { &mut MOUNT } {
        dev.info()
    } else {
        Err(())
    }
}

pub fn is_mounted() -> bool {
    unsafe { MOUNT.is_some() }
}



pub trait CharDeviceIO : Write {
    fn read(&mut self) -> Option<u8>;
    fn write(&mut self, value: u8);
}

static mut STD_OUT: Option<CharDevice> = None;
static mut STD_IN: Option<CharDevice> = None;
static mut STD_ERR: Option<CharDevice> = None;

pub fn set_stdout(dev: CharDevice) {
    unsafe {
        STD_OUT = Some(dev);
    }
}


pub fn set_stderr(dev: CharDevice) {
    unsafe {
        STD_ERR = Some(dev);
    }
}

pub fn set_stdin(dev: CharDevice) {
    unsafe {
        STD_IN = Some(dev);
    }
}


pub fn stdout<'a>() -> Option<&'a mut CharDevice> {
    unsafe {
        STD_OUT.as_mut()
    }
}

pub fn stderr<'a>() -> Option<&'a mut CharDevice> {
    unsafe {
        STD_ERR.as_mut()
    }
}

pub fn stdin<'a>() -> Option<&'a mut CharDevice> {
    unsafe {
        STD_IN.as_mut()
    }
}

pub struct Pipe {
    buffer: SharedChannel<u8>
}

impl Pipe {
    pub fn new() -> Pipe {
        Pipe { buffer: SharedChannel::new() }
    }

    pub fn write(&mut self, data: u8) {
        self.buffer.write(data)
    }

    pub fn read(&self) -> Option<u8> {
        self.buffer.read()
    }
}


pub enum CharDevice {
    Terminal,
    Serial,
    Pipe(Locked<Pipe>),
    Null
}


impl Write for CharDevice {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
        self.write(c as u8);
        }
        return Ok(());
    }
}


impl CharDeviceIO for CharDevice {
    fn read(&mut self) -> Option<u8> {
        match self {
            Self::Serial => serial::read_u8(),
            Self::Terminal => {
                let res = match input::keyboard::read_char() {
                    Some(key) => Some(key as u8),
                    None => None,
                };
                input::keyboard::clear();
                res
            },
            
            Self::Pipe(pipe) => {pipe.lock().read()},
            Self::Null => None
        }
    }

    fn write(&mut self, value: u8) {
        match self {
        Self::Pipe(pipe) => {pipe.lock().write(value)},
        Self::Terminal => terminal::print(value),
        Self::Serial => serial::write_u8(value),
        Self::Null => {},
        }
    }
}

pub mod stdout {

    pub macro println {
        ($fmt:expr, $($args:tt)*) => {
            $crate::device::stdout::write_fmt(format_args!(concat!($fmt, "\n"), $($args)*));
        },

        ($fmt:expr) => {
            $crate::device::stdout::write_fmt(format_args!(concat!($fmt, "\n")));
        }
    }

    pub macro print {
        ($fmt:expr, $($args:tt)*) => {
            $crate::device::stdout::write_fmt(format_args!($fmt, $($args)*));
        },

        ($fmt:expr) => {
            $crate::device::stdout::write_fmt(format_args!($fmt));
        }
    }

    use core::fmt::{Arguments, Write};

    use super::CharDeviceIO;

    pub fn write(chr: char) {
        if let Some(dev) = super::stdout() {
            dev.write(chr as u8)
        }
    }


    pub fn write_fmt(args: Arguments) {
        if let Some(dev) = super::stdout() {
            dev.write_fmt(args).expect("Failed To Write To Stdout.");
        }
    }
}

pub mod stderr {
    use core::fmt::{Arguments, Write};

    pub macro println {
        ($fmt:expr, $($args:tt)*) => {
            $crate::device::stderr::write_fmt(format_args!(concat!($fmt, "\n"), $($args)*));
        },

        ($fmt:expr) => {
            $crate::device::stderr::write_fmt(format_args!(concat!($fmt, "\n")));
        }
    }

    pub macro print {
        ($fmt:expr, $($args:tt)*) => {
            $crate::device::stderr::write_fmt(format_args!($fmt, $($args)*));
        },

        ($fmt:expr) => {
            $crate::device::stderr::write_fmt(format_args!($fmt));
        }
    }

    use super::CharDeviceIO;
    pub fn write(chr: char) {
        if let Some(dev) = super::stderr() {
            dev.write(chr as u8)
        }
    }

    pub fn write_fmt(args: Arguments) {
        if let Some(dev) = super::stderr() {
            dev.write_fmt(args).expect("Failed To Write To Stderr");
        }
    }
}

pub mod stdin {
    use super::CharDeviceIO;
    pub fn read() -> Option<u8> {
        if let Some(dev) = super::stdout() {
            dev.read()
        } else {
            None
        }
    }


    pub fn read_into(buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for idx in 0..buffer.len() {
            match read() {
                Some(byte) => buffer[idx] = byte,
                None => {return count;}
            }
            count += 1;
        }
        buffer.len()
    }

}


impl Display for DeviceInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Device: {} - {} Blocks", self.name, self.blocks)
    }
}