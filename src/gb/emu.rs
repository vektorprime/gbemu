use crate::gb::cpu;
use crate::gb::ram;
use crate::gb::file;

use super::bios;

pub struct Emu {
    cpu: cpu::Cpu,
    mem: ram::Ram,
    rom: Option<file::Rom>,
}

impl Emu {
    pub fn new(color_mode: bios::ColorMode) -> Self {
        Emu {
            cpu: cpu::Cpu::new(),
            mem: ram::Ram::new(color_mode),
            rom: None,
        }
    }
    pub fn load_rom(&mut self, file: String) {
        self.rom = Some(file::Rom::new(file.as_str()));
    }

    pub fn run(&mut self) {
       self.cpu.run(&mut self.mem);
        
    }

}