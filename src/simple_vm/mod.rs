
pub struct Interpreter<'a, const MEM_SIZE: usize, const STACK_SIZE: usize> {
    ip: usize,
    sp: usize,
    mem: [u8; MEM_SIZE],
    stack: [u8; STACK_SIZE],
    program: &'a [u8],
}

impl<'a, const MEM_SIZE: usize, const STACK_SIZE: usize> Interpreter<'a, MEM_SIZE, STACK_SIZE> {
    pub fn new(program: &'a [u8]) -> Self {
        Self {
            ip: 0,
            mem: [0xCC; MEM_SIZE],
            stack: [0x00; STACK_SIZE],
            program,
            sp: 0,
        }
    }

    pub fn exec(&mut self, cycles: usize) {
        for _ in 0..=cycles {
            self.clock();
        }
    }

    fn clock(&mut self) {
        match self.next_prg_byte() {
            0x10 => {self.dup_u8()}

            0x20 => {let x = self.next_prg_byte(); self.push_u8(x)}
            _ => {}
        }

        self.ip += 1;
    }

    /// 10H - Takes The First Byte Of The Stack, Duplicates It, Pushes Both Values To The Stack
    fn dup_u8(&mut self) {
        if let Some(value) = self.pop_u8() {
            self.push_u8(value);
            self.push_u8(value);
        }
    }

    /// 30H: [rhs][lhs] -> [lhs + rhs]
    fn iadd(&mut self) {
        let rhs = self.pop_u8().unwrap_or_default();
        let lhs = self.pop_u8().unwrap_or_default();

        self.push_u8(lhs + rhs);
    }

    /// 31H: [rhs][lhs] -> [lhs * rhs]
    fn imul(&mut self) {
        let rhs = self.pop_u8().unwrap_or_default();
        let lhs = self.pop_u8().unwrap_or_default();

        self.push_u8(lhs * rhs);
    }

    /// 21H: Pop A Byte From The Stack.
    fn pop_u8(&mut self) -> Option<u8> {
        if self.sp > 0 {
            let value = self.stack[self.sp];
            self.sp -= 1;
            return Some(value);
        } else {
            return None;
        }
    }

    /// 20H: Push A Byte Onto The Stack
    fn push_u8(&mut self, value: u8) {
        self.sp += 1;
        if self.sp < STACK_SIZE {
            self.stack[self.sp] = value;
        }
    }

    fn pop_u16(&mut self) -> Option<u16> {
        if self.sp > 1 {
            let h = self.stack[self.sp] as u16;
            let l = self.stack[self.sp - 1] as u16;
            self.sp -= 2;
            return Some((h << 8) | l);
        } else {
            return None;
        }
    }

    fn read_prg_u8(&mut self, index: usize) -> u8 {
        self.program[index]
    }

    fn read_prg_u16(&mut self, index: usize) -> u16 {
        let h = self.read_prg_u8(index) as u16;
        let l = self.read_prg_u8(index) as u16;

        (h << 8) | l
    }


    fn next_prg_byte(&mut self) -> u8 {
        let r = self.program[self.ip];
        self.ip += 1;

        r
    }

}