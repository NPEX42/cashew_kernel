use alloc::{collections::BTreeMap, string::String};

#[derive(Debug, Clone)]
pub enum EntryKind {
    Unused, Data, MetaData, Generic(u8),
    Super, Bitmap, Fat, Bootloader, Reserved,
}

#[derive(Debug, Clone)]
pub struct DiskEntry {
    kind: EntryKind,
    start: u32,
    size: u32, 
}

impl DiskEntry {
    pub fn new(kind: EntryKind, start: u32, size: u32) -> Self {
        Self {
            kind,
            size,
            start
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskMap {
    map: BTreeMap<String, DiskEntry>
}

impl DiskMap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new()
        }
    }

    fn add_entry(&mut self, name: &str, entry_data: DiskEntry) {
        self.map.insert(name.into(), entry_data);
    }

    fn get_entry(&mut self, name: &str) -> Option<&DiskEntry> {
        self.map.get(name.into())
    }


    
    pub fn set_bootloader(&mut self, bytes: u32) {
        self.add_entry("__bootloader", DiskEntry::new(EntryKind::Bootloader, 0, bytes / 512));
    }

    pub fn min_disk_len(&self) -> usize {
        let mut size = 0;

        for (_, entry) in &self.map {
            size += entry.size;
        }

        size as usize
    }

    pub fn add_data(&mut self, bytes: usize) {
        let start = self.min_disk_len() as u32;
        self.add_entry("__data", DiskEntry::new(EntryKind::Data, start, (bytes / 512) as u32));
    }

    pub fn set_superblock(&mut self, size: u32) {
        let start = self.min_disk_len() as u32;
        self.add_entry("__superblock", DiskEntry::new(EntryKind::Super, start, size))
    }

    pub fn set_bitmap(&mut self, records: Option<u32>) {
        let start = self.min_disk_len() as u32;
        if let Some(data_entry) = self.get_entry("__data").cloned() {
            self.add_entry("__bitmap", DiskEntry::new(EntryKind::Bitmap, start,  (data_entry.size / 8) / 512));
        } else {
            assert!(records.is_some());
            self.add_entry("__bitmap", DiskEntry::new(EntryKind::Bitmap, start, (records.unwrap() / 8) / 512));
        }
    }


    pub fn ustar_map() -> Self {
        let mut map = Self::new();

        map.add_data(1 << 20);

        map
    }

    pub fn cashew_fs_map() -> Self {
        let mut map = Self::new();
        map.set_bootloader(1 << 1);
        map.set_superblock(2);
        map.add_data(16 << 24);
        map.set_bitmap(None);

        map
    }
}
