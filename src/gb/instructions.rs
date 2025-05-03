
use crate::gb::cpu;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FlagBits {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8, // the instruction in u8
    pub name: &'static str, // name of the instruction for easy reading
    pub cycles: u8,
    pub size: u8, // some instructions are in more than one byte
    pub flags: &'static [FlagBits], // which flags were modified
}
