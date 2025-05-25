use crate::gb::cpu::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::mbc::*;
 

pub struct Emu {
    cpu: Cpu,
    bios: Bios,
    mbc: Mbc, // mbc includes rom and ram
}

impl Emu {
    pub fn new(color_mode: ColorMode) -> Self {
        Emu {
            cpu: Cpu::new(),
            mbc: Mbc::new(), // mbc has rom and ram
            bios: Bios::new(color_mode), 
        }
    }

    pub fn load_rom_file(&mut self, file: String) {
        self.mbc.rom = Some(Rom::new(file.as_str()));
    }

    pub fn init(&mut self) {
        self.mbc.ram.load_bios_to_mem(&self.bios);
        self.cpu.run_bios(&mut self.mbc, &self.bios);
        self.mbc.load_rom_to_mem();
        //self.cpu.run_rom(&mut self.mbc);
    }

    pub fn tick_cpu(&mut self) {
        self.cpu.run_rom(&mut self.mbc);
    }

}