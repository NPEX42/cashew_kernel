use core::fmt::Display;

use x64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

use crate::{
    arch::{self, *},
    sprint,
};

const PS2_DATA: u16 = 0x60;
const PS2_COMMAND: u16 = 0x64;

pub type PS2Result<T> = Result<T, &'static str>;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Command {
    DisablePort1 = 0xAD,
    DisablePort2 = 0xA7,

    EnablePort1 = 0xAE,
    EnablePort2 = 0xA8,

    TestPort1 = 0xAB,
    TestPort2 = 0xA9,

    ReadConfig = 0x20,
    WriteConfig = 0x60,

    ReadOutput = 0xD0,

    WritePort1 = 0xD2,
    WritePort2 = 0xD4,
    ReadPort2 = 0xD3,

    SelfTest = 0xAA,
}

impl Into<u8> for Command {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Into<usize> for Command {
    fn into(self) -> usize {
        self as usize
    }
}

pub struct PS2Controller {
    status: PortReadOnly<u8>,
    command: PortWriteOnly<u8>,

    data: Port<u8>,
}

impl PS2Controller {
    pub fn get() -> Self {
        Self {
            command: PortWriteOnly::new(PS2_COMMAND),
            status: PortReadOnly::new(PS2_COMMAND),

            data: Port::new(PS2_DATA),
        }
    }

    pub fn status(&mut self) -> Status {
        unsafe { Status::from(self.status.read()) }
    }

    pub fn write_data(&mut self, data: u8) -> PS2Result<()> {
        self.wait_for_write()?;
        unsafe { self.data.write(data) }
        Ok(())
    }

    pub fn read_data(&mut self) -> PS2Result<u8> {
        self.wait_for_read()?;
        unsafe { Ok(self.data.read()) }
    }

    pub fn input_ready(&mut self) -> bool {
        self.status().is_set(StatusFlags::InputBufferFull)
    }

    pub fn output_ready(&mut self) -> bool {
        self.status().is_clear(StatusFlags::OutputBufferFull)
    }

    pub fn command(&mut self, cmd: Command) -> PS2Result<()> {
        self.wait_for_write()?;
        unsafe { self.command.write(cmd as u8) }
        Ok(())
    }

    pub fn wait_for_write(&mut self) -> PS2Result<()> {
        self.wait_on_status_clear(StatusFlags::InputBufferFull)
    }

    pub fn wait_for_read(&mut self) -> PS2Result<()> {
        self.wait_on_status_set(StatusFlags::OutputBufferFull)?;
        Ok(())
    }

    pub fn wait_on_status_set(&mut self, flag: StatusFlags) -> PS2Result<()> {
        for _ in 0..3 {
            if self.status().is_set(flag) {
                return Ok(());
            };
            spin();
        }
        Err("Status Set Timeout")
    }

    pub fn wait_on_status_clear(&mut self, flag: StatusFlags) -> PS2Result<()> {
        for _ in 0..3 {
            if self.status().is_clear(flag) {
                return Ok(());
            };
            spin();
        }
        Err("Status Clear Timeout")
    }

    pub fn reinit(&mut self) -> PS2Result<()> {
        arch::disable_interrupts();

        // Step 3 - Disable Devices
        sprint!("[PS/2]: Disabling Devices\n");
        self.command(Command::DisablePort1)?;
        self.command(Command::DisablePort2)?;

        // Step 4 - Flush Output Buffer
        sprint!("[PS/2]: Flushing Output Buffer\n");
        #[allow(unused_must_use)]
        {
            self.read_data();
        }

        // Step 5 - Disable Port IRQs & Translation
        sprint!("[PS/2]: Disabling IRQs & Translation\n");
        self.command(Command::ReadConfig)?;
        let mut cfg = Config::from(self.read_data()?);
        sprint!("[PS/2]: Config - {}\n", cfg);
        cfg.clear(ConfigFlags::Port1IrqEnabled);
        cfg.clear(ConfigFlags::Port2IrqEnabled);
        cfg.clear(ConfigFlags::Port1TranslateEn);

        sprint!("[PS/2]: Config - {}\n", cfg);
        self.command(Command::WriteConfig)?;
        self.write_data(cfg.as_u8())?;

        sprint!("[PS/2]: Self Testing\n");
        self.command(Command::SelfTest)?;
        assert!(self.read_data()? == 0x55);

        self.command(Command::TestPort1)?;
        assert!(self.read_data()? == 0x00);

        self.command(Command::TestPort2)?;
        assert!(self.read_data()? == 0x00);

        sprint!("[PS/2]: Enabling Devices\n");
        self.command(Command::EnablePort1)?;
        self.command(Command::EnablePort2)?;

        self.command(Command::ReadConfig)?;
        let mut cfg = Config::from(self.read_data()?);
        cfg.set(ConfigFlags::Port1IrqEnabled);
        cfg.set(ConfigFlags::Port2IrqEnabled);
        cfg.set(ConfigFlags::Port1TranslateEn);

        self.command(Command::WriteConfig)?;
        self.write_data(cfg.as_u8())?;
        arch::enable_interrupts();

        Ok(())
    }

    pub fn read_port_2(&mut self) -> PS2Result<u8> {
        self.command(Command::ReadPort2)?;
        self.read_data()
    }

    pub fn write_port_2(&mut self, data: u8) -> PS2Result<()> {
        self.command(Command::WritePort2)?;
        self.write_data(data)
    }

    pub fn is_dual_channel(&mut self) -> PS2Result<bool> {
        self.command(Command::ReadConfig)?;
        let config_old = Config::from(self.read_data()?);
        self.command(Command::DisablePort2)?;
        let cfg = Config::from(self.read_data()?);
        let result = config_old.as_u8() != cfg.as_u8();
        self.command(Command::WriteConfig)?;
        self.write_data(config_old.as_u8())?;
        Ok(result)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum StatusFlags {
    OutputBufferFull = 1 << 0,
    InputBufferFull = 1 << 1,
    SystemFlag = 1 << 2,
    ControllerData = 1 << 3,
    KeyboardLock = 1 << 4,
    RXTimeout = 1 << 5,
    TimeOutError = 1 << 6,
    ParityError = 1 << 7,
}

pub struct Status {
    status: u8,
}

impl Status {
    pub fn from(flags: u8) -> Self {
        Self { status: flags }
    }

    pub fn is_set(&self, flag: StatusFlags) -> bool {
        (self.status & flag as u8) > 0
    }

    pub fn is_clear(&self, flag: StatusFlags) -> bool {
        (self.status & flag as u8) == 0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ConfigFlags {
    Port1IrqEnabled = 1 << 0,
    Port2IrqEnabled = 1 << 1,
    PassedPost = 1 << 2,
    SBZ = 1 << 3,
    Port1ClockDisable = 1 << 4,
    Port2ClockDisable = 1 << 5,
    Port1TranslateEn = 1 << 6,
    MBZ = 1 << 7,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Config {
    config: u8,
}

impl Config {
    pub fn as_u8(&self) -> u8 {
        self.config
    }

    pub fn from(flags: u8) -> Self {
        Self { config: flags }
    }

    pub fn is_set(&self, flag: ConfigFlags) -> bool {
        self.config & flag as u8 > 0
    }

    pub fn is_clear(&self, flag: ConfigFlags) -> bool {
        self.config & flag as u8 == 0
    }

    pub fn set(&mut self, flag: ConfigFlags) {
        self.config |= flag as u8
    }

    pub fn clear(&mut self, flag: ConfigFlags) {
        self.config &= !(flag as u8)
    }

    pub fn toggle(&mut self, flag: ConfigFlags) {
        self.config ^= flag as u8
    }

    pub fn bits(&self) -> [bool; 8] {
        let mut bits = [false; 8];
        for bit in 0..bits.len() {
            bits[bit] = self.config & (0x80 >> bit) > 0;
        }
        bits
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bits = self.bits();

        if bits[1] {
            write!(f, "TRANSLATE | ")?
        };
        if bits[2] {
            write!(f, "PORT 2 CLK | ")?
        };
        if bits[3] {
            write!(f, "PORT 1 CLK | ")?
        };
        if bits[5] {
            write!(f, "PASSED POST | ")?
        };
        if bits[6] {
            write!(f, "PORT 2 IRQ | ")?
        };
        if bits[7] {
            write!(f, "PORT 1 IRQ")?
        };
        write!(f, "")
    }
}
