pub struct Fuse(bool);

impl Fuse {
    pub const fn new() -> Self {
        Self(false)
    }

    pub fn test(&mut self) {
        if self.0 {
            panic!("Fuse Tripped!")
        } else {
            self.0 = true;
        }
    }
}
