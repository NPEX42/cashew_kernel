use alloc::{vec::Vec};

static mut OPCODES: Vec<&'static str> = Vec::new();


pub unsafe fn init() {
    
        OPCODES.clear();

        OPCODES.push("pushi");
        OPCODES.push("pushs");

        OPCODES.push("pop");
        OPCODES.push("load");
        OPCODES.push("store");

        OPCODES.push("dup");
        OPCODES.push("swap");

        OPCODES.push("add");
        OPCODES.push("sub");
        OPCODES.push("div");
        OPCODES.push("mul");
        OPCODES.push("mod");

        OPCODES.push("and");
        OPCODES.push("xor");
        OPCODES.push("or");
        OPCODES.push("not");
        OPCODES.push("shr");
        OPCODES.push("shl");

        OPCODES.push("jmp");
        OPCODES.push("jmz");
        OPCODES.push("jnz");
}

pub fn is_opcode(lexeme: &str) -> bool {
    unsafe {
        OPCODES.contains(&lexeme)
    }
}