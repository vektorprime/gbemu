use crate::gb::cpu::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::mbc::*;
 use crate::gb::graphics::ppu::*;
 use crate::gb::graphics::lcd::*;
use crate::gb::hwregisters::HardwareRegisters;

use std::time::{Duration, Instant};

pub struct Emu {
    pub cpu: Cpu,
    bios: Bios,
    pub mbc: Box<Mbc>, // mbc includes rom and ram
    pub ppu: Ppu,
    // pub lcd: Lcd,
    pub debug: bool,
    pub sec_cycles: u64, // tracking max mcycles per sec
    pub current_time: Instant,
}

impl Emu {
    pub fn new(color_mode: ColorMode, debug: bool) -> Self {
        Emu {
            cpu: Cpu::new(),
            mbc: Box::new(Mbc::new()), // mbc has rom and ram
            bios: Bios::new(color_mode), 
            ppu: Ppu::new(),
            // lcd: Lcd::new(),
            debug,
            sec_cycles: 0, // tracking max mcycles per sec
            current_time: Instant::now(),
        }
    }

    pub fn load_rom_file(&mut self, file: String) {
        self.mbc.rom = Some(Rom::new(file.as_str()));
    }

    pub fn load_bios(&mut self) {
        self.mbc.boot_rom.load_bios_to_mem(&self.bios);
    }

    // pub fn init_ppu(&mut self) {
    //     self.ppu.load_all_tiles(&self.mbc);
    // }


    pub fn tick(&mut self, tile_frame: &mut [u8], game_frame: &mut [u8]) -> RenderState {
        let mcycle_per_sec: u64 = 17556;
        let one_sec: u64 = 1;
        if self.current_time.elapsed().as_secs() < one_sec {
            if self.sec_cycles < mcycle_per_sec {
                let cycles = self.cpu.tick(&mut self.mbc, &self.bios);
                self.sec_cycles += cycles;
                self.ppu.tick(&mut self.mbc, tile_frame, game_frame, cycles)
            } else {
                RenderState::NoRender
            }
        }   else {
            print!("sec has elapsed\n");
            self.sec_cycles = 0;
            self.current_time = Instant::now();
            RenderState::NoRender
        }
        // if self.sec_cycles < mcycle_per_sec && self.current_time.elapsed().as_secs() < one_sec {
        //     let cycles = self.cpu.tick(&mut self.mbc, &self.bios);
        //     self.sec_cycles += cycles;
        //     self.ppu.tick(&mut self.mbc, tile_frame, game_frame, cycles)
        // }



    }

}