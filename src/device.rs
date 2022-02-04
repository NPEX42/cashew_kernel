use alloc::string::String;

use crate::ata;

pub type BlockAddr = u32;

static mut MOUNT: Option<Device> = None;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub blocks: usize,
    pub name: String,
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

    fn block_count(&self) -> Result<usize, ()>;

    fn info(&self) -> Result<DeviceInfo, ()> {
        Ok(DeviceInfo::generic_device(self.block_count()?))
    }
}

pub enum Device {
    Ata(u8, u8),
}

impl BlockDeviceIO for Device {
    fn block_count(&self) -> Result<usize, ()> {
        match self {
            Device::Ata(bus, drive) => ata::get_sector_count(*bus, *drive),
        }
    }

    fn read(&self, block: BlockAddr) -> Result<[u8; ata::BLOCK_SIZE], ()> {
        match self {
            Device::Ata(bus, drive) => ata::read(*bus, *drive, block),
        }
    }

    fn write(&mut self, block: BlockAddr, data: &[u8]) -> Result<(), ()> {
        match self {
            Device::Ata(bus, drive) => ata::write(*bus, *drive, block, data),
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

pub fn write(block: BlockAddr, data: &[u8]) -> Result<(), ()> {
    if let Some(dev) = unsafe { &mut MOUNT } {
        dev.write(block, data)
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
