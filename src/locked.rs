use spinning_top::{Spinlock, SpinlockGuard};

pub struct Locked<T> {
    item: Spinlock<T>
}

impl<T> Locked<T> {
    pub fn new(item: T) -> Self {
        Self {
            item: Spinlock::new(item)
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

