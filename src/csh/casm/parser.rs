use alloc::string::String;

pub enum Node {
    Integer(isize),
    Identifier(String),
    Opcode()
}

pub enum Opcode {
    PushImm(Node),
    PushStr(Node),
    Pop,
    Load,
    Store,
    Duplicate,
    Swap,
    Add,
    Subtract,
    Multiply,
    Divide,
}
