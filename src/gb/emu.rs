use crate::gb::cpu::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::mbc::*;
 use crate::gb::graphics::ppu::*;

pub struct Emu {
    cpu: Cpu,
    bios: Bios,
    mbc: Mbc, // mbc includes rom and ram
    ppu: Ppu,
}

impl Emu {
    pub fn new(color_mode: ColorMode) -> Self {
        Emu {
            cpu: Cpu::new(),
            mbc: Mbc::new(), // mbc has rom and ram
            bios: Bios::new(color_mode), 
            ppu: Ppu::new(),
        }
    }

    pub fn load_rom_file(&mut self, file: String) {
        self.mbc.rom = Some(Rom::new(file.as_str()));
    }

    pub fn load_bios(&mut self) {
        self.mbc.ram.load_bios_to_mem(&self.bios);
    }

    pub fn init_ppu(&mut self) {
        self.ppu.load_all_tiles(&self.mbc);
    }

    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.mbc, &self.bios);
        self.ppu.tick(&self.mbc);
    }

}