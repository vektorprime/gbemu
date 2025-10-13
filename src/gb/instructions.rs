


#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8, // the instruction in u8
    pub name: &'static str, // name of the instruction for easy reading
    pub cycles: u8,
    pub size: u8, // some instructions are in more than one byte

}
