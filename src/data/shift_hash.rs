

use alloc::{string::String, vec};
use vec::*;

use crate::{csh, println, sprint};

const BLOCK_SIZE: usize = 16;

const PADDING_VAL: u8 = 0xAA;

pub fn hash_u16(data: &[u8]) -> u16 {
    let mut result = 0;
    let mut blocks = data.len() / BLOCK_SIZE;
    if data.len() % BLOCK_SIZE > 0 {blocks += 1};

    let padding: Vec<u8> = vec![PADDING_VAL; (blocks * BLOCK_SIZE) - data.len()];



    for byte in data {
        result ^= (*byte) as u16;
        result = result.rotate_left((result & 0b11) as u32);
        sprint!("result (u16): {}\n", result)
    }

    sprint!("Padding Bytes: {}\n", padding.len());

    for byte in padding {
        result ^= byte as u16;
        result = result.rotate_left((result & 0b11) as u32);
    }

    


    result
}

pub fn hash_u32(data: &[u8]) -> u32 {
    let mut result = 0;

    let mut blocks = data.len() / BLOCK_SIZE;
    if data.len() % BLOCK_SIZE > 0 {blocks += 1};

    let padding: Vec<u8> = vec![PADDING_VAL; (blocks * BLOCK_SIZE) - data.len()];

    for byte in data {
        result ^= (*byte) as u32;
        result = result.rotate_left(result & 0b11);
        sprint!("result (u32): {}\n", result)
    }

    sprint!("Padding Bytes: {}\n", padding.len());

    for byte in padding {
        result ^= byte as u32;
        result = result.rotate_left((result & 0b11) as u32);
        
    }

    result
}

pub fn hash_u64(data: &[u8]) -> u64 {
    let mut result = 0;

    let mut blocks = data.len() / BLOCK_SIZE;
    if data.len() % BLOCK_SIZE > 0 {blocks += 1};

    let padding: Vec<u8> = vec![PADDING_VAL; (blocks * BLOCK_SIZE) - data.len()];

    for byte in data {
        result ^= (*byte) as u64;
        result = result.rotate_left((result & 0b11) as u32);
        sprint!("result (u64): {}\n", result)
    }

    sprint!("Padding Bytes: {}\n", padding.len());

    for byte in padding {
        result ^= byte as u64;
        result = result.rotate_left((result & 0b11) as u32);
    }

    result
}

pub fn hash_u128(data: &[u8]) -> u128 {
    let mut result = 0;

    let mut blocks = data.len() / BLOCK_SIZE;
    if data.len() % BLOCK_SIZE > 0 {blocks += 1};
    let padding: Vec<u8> = vec![0xCC; (blocks * BLOCK_SIZE) - data.len()];
    for byte in data {
        result ^= (*byte) as u128;
        result = result.rotate_left((result & 0b11) as u32);


        sprint!("result (u128): {}\n", result)
    }

    sprint!("Padding Bytes: {}\n", padding.len());

    for byte in padding {
        result ^= byte as u128;
        result = result.rotate_left((result & 0b11) as u32);
    }

    result
}

pub fn main(args: Vec<String>) -> csh::ExitCode {
    if args.len() < 3 {
        println!("usage: {} <Size: 16, 32> <item to hash>", args[0]);
        return csh::ExitCode::Error(1);
    }

    let size = &args[1];
    let item = &args[2..].join(" ");

    let hash = match size.as_str() {
        "16" =>  {hash_u16 (item.as_bytes()) as u128},
        "32" =>  {hash_u32 (item.as_bytes()) as u128},
        "64" =>  {hash_u64 (item.as_bytes()) as u128},
        "128" => {hash_u128(item.as_bytes()) as u128},
        _ => {println!("usage: {} <Size: 16, 32, 64, 128> <item to hash>", args[0]); return csh::ExitCode::Error(2)}
    };
    let size: usize = size.parse().unwrap();
    println!("hash_u{}({}) = ${:0width$x}", size, item, hash, width = (size / 4));

    csh::ExitCode::Ok
}