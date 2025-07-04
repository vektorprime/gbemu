use crate::gb::bios::*;



pub struct Ram {
    pub memory: Vec<u8>,
}


impl Ram {
    pub fn new() -> Self {
        Ram {
            memory: vec![0; 65536],
        }
    }

    pub fn load_bios_to_mem(&mut self, bios: &Bios) {
        for (i, byte) in bios.data.iter().copied().enumerate() {
            self.memory[i] = byte;
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }
}


pub struct RomRam {
    pub memory: [u8; 131071],
}


impl RomRam {
    pub fn new() -> Self {
        RomRam {
            memory: [0; 131071],
        }
    }

    pub fn read(&self, address: u32) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u32, byte: u8) {
        self.memory[address as usize] = byte;
    }
}