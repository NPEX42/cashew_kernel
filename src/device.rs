use core::{ops::Range, fmt::Write};

use alloc::{string::String, boxed::Box};

use crate::{
    ata,
    csh::{ErrorCode, ExitCode, ShellArgs},
    println, sprint,
    vfs::block::Block, serial, terminal, input,
};

pub type BlockAddr = u32;

static mut MOUNT: Option<Device> = None;

pub fn mount_main(args: ShellArgs) -> ExitCode {
    if args.len() < 2 {
        println!("Usage: {} [hda|hdb|hdc|hdd]", args[0]);
        return ExitCode::Error(ErrorCode::Usage);
    }

    if let Some(dev) = Device::from_str(&args[1]) {
        mount(dev)
    } else {
        println!("Invalid Device: '{}'", args[1]);
        println!("Usage: {} [hda|hdb|hdc|hdd]", args[0]);
        return ExitCode::Error(ErrorCode::Usage);
    }

    ExitCode::Ok
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

pub enum Device {
    Ata(u8, u8),
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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hda" => Some(Self::hda()),
            "hdb" => Some(Self::hdb()),
            "hdd" => Some(Self::hdc()),
            "hdc" => Some(Self::hdd()),
            _ => None,
        }
    }
}

impl BlockDeviceIO for Device {
    fn block_count(&self) -> Result<usize, ()> {
        match self {
            Device::Ata(bus, drive) => ata::get_sector_count(*bus, *drive),
        }
    }

    fn read(&self, block: BlockAddr) -> Result<[u8; ata::BLOCK_SIZE], ()> {
        match self {
            Device::Ata(bus, drive) => ata::read_block(*bus, *drive, block),
        }
    }

    fn write(&mut self, block: BlockAddr, data: &[u8]) -> Result<(), ()> {
        match self {
            Device::Ata(bus, drive) => ata::write_block(*bus, *drive, block, data),
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


pub enum CharDevice {
    Terminal,
    Serial,
    Pipe,
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
            
            Self::Pipe => {None}
        }
    }

    fn write(&mut self, value: u8) {
        match self {
        Self::Pipe => {},
        Self::Terminal => terminal::print(value),
        Self::Serial => serial::write_u8(value)
        }
    }
}

pub mod stdout {
    use core::fmt::{Arguments, Write};

    use super::CharDeviceIO;

    pub fn write(chr: char) {
        if let Some(dev) = super::stdout() {
            dev.write(chr as u8)
        }
    }


    pub fn write_fmt(args: Arguments) {
        if let Some(dev) = super::stdout() {
            dev.write_fmt(args);
        }
    }
}

pub mod stderr {
    use core::fmt::{Arguments, Write};

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