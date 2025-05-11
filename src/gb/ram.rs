use crate::gb::bios::*;



pub struct Ram {
    memory: [u8; 65536],
    bios:  Bios,
}


impl Ram {
    pub fn new(mode: ColorMode) -> Self {
        let bios = Bios::new(mode);

        let mut mem = Ram {
            memory: [0; 65536],
            bios,
        };

        mem.load_bios_to_mem();

        mem
    }

    pub fn load_bios_to_mem(&mut self) {
        for (i, byte) in self.bios.data.iter().copied().enumerate() {
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