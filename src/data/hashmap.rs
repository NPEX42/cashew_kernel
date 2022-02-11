use alloc::string::String;

use crate::sprint;

/// Default Bucket Size, Size - 2KB
pub const BUCKET_COUNT: usize = 256;

pub struct HashMap<V: Copy>  where V: core::marker::Copy {
    buckets: [Option<V>; BUCKET_COUNT],
}

impl<V> HashMap<V> where V: Copy {
    pub const fn new() -> Self {
        Self {
            buckets: [None; BUCKET_COUNT]
        }
    }

    pub fn insert(&mut self, key: &String,  value: V) -> bool {
        let hash = super::shift_hash::hash_u16(key.as_bytes());

        sprint!("'{}' --(hash)--> {:04X}\n", key ,hash);
        let index = hash as usize & (BUCKET_COUNT - 1);
        if self.buckets[index].is_none() {
            self.buckets[index] = Some(value);
            return true;
        }

        return false;
    }

    pub fn get(&self, key: &String) -> Option<V> {
        let hash = super::shift_hash::hash_u16(key.as_bytes()) % BUCKET_COUNT as u16;

        self.buckets[hash as usize]
    }
}

