use core::ptr::NonNull;

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use alloc::boxed::Box;
use aml::{AmlContext, Handler};
use x86_64::{instructions::port::Port, PhysAddr};

use crate::{klog, mem};

#[allow(dead_code)]
#[repr(u64)]
enum FADT {
    SciInterrupt = 46,     // u16,
    SmiCmdPort = 48,       // u32,
    AcpiEnable = 52,       // u8,
    AcpiDisable = 53,      // u8,
    S4biosReq = 54,        // u8,
    PstateControl = 55,    // u8,
    Pm1aEventBlock = 56,   // u32,
    Pm1bEventBlock = 60,   // u32,
    Pm1aControlBlock = 64, // u32,
    Pm1bControlBlock = 68, // u32,
}

fn read_addr<T>(physical_address: usize) -> T
where
    T: Copy,
{
    let virtual_address = crate::mem::phys_to_virt(PhysAddr::new(physical_address as u64)).unwrap();
    unsafe { *virtual_address.as_ptr::<T>() }
}

fn read_fadt<T>(address: usize, offset: FADT) -> T
where
    T: Copy,
{
    read_addr::<T>(address + offset as usize)
}

pub fn shutdown() {
    let mut pm1a_control_block = 0;
    let mut slp_typa = 0;
    let slp_len = 1 << 13;

    let mut aml = AmlContext::new(Box::new(CashewAmlHandler), aml::DebugVerbosity::None);
    let res = unsafe { AcpiTables::search_for_rsdp_bios(CashewAcpiHandler) };

    match res {
        Ok(acpi) => {
            for (sig, sdt) in acpi.sdts {
                if sig.as_str() == "FACP" {
                    klog!("FACP Found\n");
                    pm1a_control_block =
                        read_fadt::<u32>(sdt.physical_address, FADT::Pm1aControlBlock);
                }
            }

            match &acpi.dsdt {
                Some(dsdt) => {
                    let address = mem::phys_to_virt(PhysAddr::new(dsdt.address as u64)).unwrap();
                    let stream = unsafe {
                        core::slice::from_raw_parts(address.as_ptr(), dsdt.length as usize)
                    };

                    if aml.parse_table(stream).is_ok() {
                    } else {
                        klog!("ACPI Failed to parse AML in DSDT\n");
                        // FIXME: AML parsing works on QEMU and Bochs but not
                        // on VirtualBox at the moment, so we use the following
                        // hardcoded value:
                        slp_typa = (5 & 7) << 10;
                    }
                }

                None => {}
            }
        }

        Err(_e) => {
            klog!("Failed To Read RSDP Table\n");
        }
    }

    let mut port: Port<u16> = Port::new(pm1a_control_block as u16);
    unsafe {
        port.write(slp_typa | slp_len);
    }
}

#[derive(Clone)]
pub struct CashewAcpiHandler;

impl AcpiHandler for CashewAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address =
            crate::mem::phys_to_virt(PhysAddr::new(physical_address as u64)).unwrap();
        PhysicalMapping::new(
            physical_address,
            NonNull::new(virtual_address.as_mut_ptr()).unwrap(),
            size,
            size,
            Self,
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

struct CashewAmlHandler;

impl Handler for CashewAmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        read_addr::<u8>(address)
    }
    fn read_u16(&self, address: usize) -> u16 {
        read_addr::<u16>(address)
    }
    fn read_u32(&self, address: usize) -> u32 {
        read_addr::<u32>(address)
    }
    fn read_u64(&self, address: usize) -> u64 {
        read_addr::<u64>(address)
    }
    fn write_u8(&mut self, _address: usize, _value: u8) {
        unimplemented!()
    }
    fn write_u16(&mut self, _address: usize, _value: u16) {
        unimplemented!()
    }
    fn write_u32(&mut self, _address: usize, _value: u32) {
        unimplemented!()
    }
    fn write_u64(&mut self, _address: usize, _value: u64) {
        unimplemented!()
    }
    fn read_io_u8(&self, _port: u16) -> u8 {
        unimplemented!()
    }
    fn read_io_u16(&self, _port: u16) -> u16 {
        unimplemented!()
    }
    fn read_io_u32(&self, _port: u16) -> u32 {
        unimplemented!()
    }
    fn write_io_u8(&self, _port: u16, _value: u8) {
        unimplemented!()
    }
    fn write_io_u16(&self, _port: u16, _value: u16) {
        unimplemented!()
    }
    fn write_io_u32(&self, _port: u16, _value: u32) {
        unimplemented!()
    }
    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 {
        unimplemented!()
    }
    fn read_pci_u16(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
    ) -> u16 {
        unimplemented!()
    }
    fn read_pci_u32(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
    ) -> u32 {
        unimplemented!()
    }
    fn write_pci_u8(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u8,
    ) {
        unimplemented!()
    }
    fn write_pci_u16(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u16,
    ) {
        unimplemented!()
    }
    fn write_pci_u32(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u32,
    ) {
        unimplemented!()
    }
}
