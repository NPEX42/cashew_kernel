use alloc::collections::VecDeque;
use spinning_top::{Spinlock, SpinlockGuard};

pub struct Locked<T> {
    item: Spinlock<T>,
}

impl<T> Locked<T> {
    pub fn new(item: T) -> Self {
        Self {
            item: Spinlock::new(item),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<T> {
        self.item.lock()
    }

    pub fn force_unlock(&self) {
        unsafe {
            self.item.force_unlock();
        }
    }
}

unsafe impl<T> Send for Locked<T> {}
unsafe impl<T> Sync for Locked<T> {}




pub struct SharedChannel<T: Sync + Send> {
    buffer: Locked<VecDeque<T>>
}

impl<T: Sync + Send> SharedChannel<T> {
    pub fn new() -> Self {
        SharedChannel { buffer: Locked::new(VecDeque::new()) }
    }

    pub fn as_mut(&mut self) -> &mut Self {
        return self;
    }

    pub fn write(&self, value: T) {
        let mut buffer_guard = self.buffer.lock();
        buffer_guard.push_front(value);
    }

    pub fn read(&self) -> Option<T> {
        let mut buffer_guard = self.buffer.lock();
        buffer_guard.pop_back()
    }
}

unsafe impl<T: Sync + Send> Send for SharedChannel<T> {}
unsafe impl<T: Sync + Send> Sync for SharedChannel<T> {}
