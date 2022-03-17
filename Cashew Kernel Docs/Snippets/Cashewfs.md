```rust
const MAX_FILENAME: usize = 16;
#[repr(C, align(32))]
#[derive(Debug, Clone)]
pub struct DirEntry {
	name: [u8; MAX_FILENAME],
	kind: u8,
	flags: u8,
}

const DIR_ENTRIES: usize = 512 / core::mem::size_of::<DirEntry>();

#[repr(transparent)]
pub struct Dir {
	entries: [DIR_ENTRIES]
}
```