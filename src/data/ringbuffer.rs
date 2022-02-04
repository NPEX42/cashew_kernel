use crate::klog;

pub const MAX_ITEMS: usize = 32;

pub struct RingBuffer<T> {
    items: [(bool, T); MAX_ITEMS],

    write_index: usize,
    read_index: usize,
}

impl<T: Default + Copy> RingBuffer<T> {
    pub fn default() -> Self {
        Self {
            items: [(false, Default::default()); MAX_ITEMS],
            read_index: 0,
            write_index: 0,
        }
    }

    pub fn read(&mut self) -> Option<T> {
        let item = if self.items[self.read_index].0 {
            self.items[self.read_index].0 = false;
            let r = Some(self.items[self.read_index].1);
            self.read_index += 1;
            self.read_index %= MAX_ITEMS;
            r
        } else {
            None
        };

        item
    }

    pub fn write(&mut self, item: T) -> bool {
        klog!("Write Index: {}\n", self.write_index);
        if !self.items[self.write_index].0 {
            self.items[self.write_index].0 = true;
            self.items[self.write_index].1 = item;

            self.write_index = self.write_index + 1;
            self.write_index = self.write_index % MAX_ITEMS;
        } else {
            return false;
        }

        return true;
    }
}
