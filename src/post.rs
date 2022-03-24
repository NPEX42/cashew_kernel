use crate::{klog, vfs::block::Block};

pub fn self_test() {
    

    klog!("POST::Block::Free\n");
    let mut block = Block::allocate().unwrap();
    let first_addr = block.addr();
    block.free();
    let mut block = Block::allocate().unwrap();
    let second_addr = block.addr();
    block.free();
    assert_eq!(
        first_addr, second_addr,
        "Block::free() Failed, Allocated Different Addresses."
    );
}
