use crate::gb::registers;
use crate::gb::instructions;

pub struct Cpu {
    pub registers: registers::Registers,
    pub ime: bool, // interrupt master
    pub opcode: u8, // opcode of current inst.
    pub cycles: u64, // total cycles count
    pub valid_instructions: Vec<instructions::Instruction>,
}


impl Cpu {

    pub fn new() -> Self {
        Cpu {
            registers: registers::Registers::new(),
            ime: false,
            opcode: 0,
            cycles: 0,
            valid_instructions: Self::setup_instructions(),
        }
    }

    pub fn fetch_next_inst(&mut self) -> u8 {
        // todo
        0
    }

    pub fn decode_opcode(&self, opcode: u8, cb_opcode: bool) -> instructions::Instruction {
        instructions::Instruction {
            opcode: 0xC3,
            name: "JP",
            cycles: 4,
            size: 3,
            flags: &[],
        }
    }

    pub fn execute_instruction(&mut self,  inst: &instructions::Instruction) -> u64 {
        // todo
        0
    }

    fn setup_instructions() -> Vec<instructions::Instruction> {
        let mut all_instructions = Vec::new();

        let c3 = instructions::Instruction {
            opcode: 0xC3,
            name: "JP",
            cycles: 4,
            size: 3,
            flags: &[],
        };

        all_instructions.push(c3);

        all_instructions
    }
}