use elf_rs::{self, Elf};

pub fn parse(bin: &[u8]) -> Result<Elf, elf_rs::Error> {
    Elf::from_bytes(bin)
}
