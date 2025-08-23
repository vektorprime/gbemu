use crate::gb::cpu::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::mbc::*;
use crate::gb::graphics::ppu::*;
use crate::gb::testcpu::*;
use crate::gb::hwregisters::HardwareRegisters;
use crate::gb::gbwindow::*;

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use pixels::Pixels;

pub struct Emu {
    pub cpu: Cpu,
    bios: Bios,
    pub mbc: Box<Mbc>, // mbc includes rom and ram
    pub ppu: Ppu,
    // pub lcd: Lcd,
    pub debug: bool,
    pub sec_mcycles: u64, // tracking max mcycles per sec
    pub current_time: Instant,
    pub is_cpu_test_enabled: bool,
    pub is_cpu_tested: bool,
    pub test_mbc: Box<Mbc>,
    pub test_cpu: Cpu,
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
            sec_mcycles: 0, // tracking max mcycles per sec
            current_time: Instant::now(),
            is_cpu_tested: false,
            is_cpu_test_enabled: true,
            test_mbc: Box::new(Mbc::new()),
            test_cpu: Cpu::new(),
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

    pub fn test_cpu(&mut self) {
        println!("TESTING CPU");
        let all_cpu_tests = get_all_tests();
        //println!("Got all tests, starting testing");
        for test in &all_cpu_tests {
            //println!("executing test {}", test.name);
            // setup state
                // registers
            setup_initial_registers(&mut self.test_cpu.registers, &test.initial_test_state);
            // ram
            for ram_entry in &test.initial_test_state.test_ram {
                self.test_mbc.write(ram_entry.0, ram_entry.1, OpSource::CPU);
            }
            // setup done

            // execute cpu
            self.test_cpu.tick(&mut self.test_mbc);


            // check state
                // registers
                let failed_registers = compare_registers(&self.test_cpu.registers, &test.final_test_state);
                if !failed_registers.is_empty() {
                    println!("Failed registers:");
                    for register in &failed_registers {
                        println!("{:x?}", register);
                    }
                    panic!("CPU TEST FAILED - REGISTERS");
                }
            else {
                //println!("all registers passed!");
            }
                // ram
            // clean up registers and ram
            for ram_entry in &test.initial_test_state.test_ram {
                self.test_mbc.write(ram_entry.0, 0, OpSource::CPU);
            }
            // print result
            // if result says all CPU tests passed, continue
            // else print name, error, and panic
        }
        println!("FINISHED TESTING CPU");
    }

    //pub fn tick(&mut self, tile_frame: &mut [u8], game_frame: &mut [u8]) -> RenderState {
    pub fn tick(&mut self, tw: &Arc<Mutex<Vec<u8>>>, bgmw: &Arc<Mutex<Vec<u8>>>, gw: &Arc<Mutex<Vec<u8>>>) -> PPUEvent {
        if self.is_cpu_test_enabled && !self.is_cpu_tested {
            self.test_mbc.is_testing_enabled = true;
            self.test_cpu();
            self.is_cpu_tested = true;
            self.test_mbc.is_testing_enabled = false;
        }

        let mcycles_per_sec: u64 = 1_053_360;
        let one_sec: u64 = 1;
        let elapsed_time = self.current_time.elapsed().as_secs();
        if elapsed_time < one_sec {
            if self.sec_mcycles < mcycles_per_sec {
                let mcycles = self.cpu.tick(&mut self.mbc);
                self.sec_mcycles += mcycles;
                self.ppu.tick(&mut self.mbc, tw, bgmw, gw, mcycles)
            } else {
                return PPUEvent::RenderEvent(RenderState::NoRender);
            }
        }  else {
            if elapsed_time > one_sec && self.debug == false {
                panic!("ERROR: Elapsed time greater than one sick in EMU tic\n");
            } else {
                if self.sec_mcycles < mcycles_per_sec {
                    print!("sec has elapsed without reaching max mcycles, current mcycle is {}\n", self.sec_mcycles);
                }
                else {
                    print!("sec has elapsed and reached max mcycle\n");
                }
                self.sec_mcycles = 0;
                self.current_time = Instant::now();
                return PPUEvent::RenderEvent(RenderState::NoRender);
            }

        }

    }

    pub fn tick_no_window(&mut self) -> PPUEvent {
        let mcycle_per_sec: u64 = 1_053_360;
        let one_sec: u64 = 1;
        let elapsed_time = self.current_time.elapsed().as_secs();
        if elapsed_time < one_sec {
            if self.sec_mcycles < mcycle_per_sec {
                let cycles = self.cpu.tick(&mut self.mbc);
                self.sec_mcycles += cycles;
                PPUEvent::RenderEvent(RenderState::NoRender)
                //self.ppu.tick_no_window(&mut self.mbc, cycles)
            } else {
                PPUEvent::RenderEvent(RenderState::NoRender)
            }
        }  else {
            if elapsed_time > one_sec {
                panic!("ERROR: Elapsed time greater than one sick in EMU tick");
            } else {
                if self.sec_mcycles < mcycle_per_sec {
                    print!("sec has elapsed without reaching max mcycles, current mcycle is {}\n", self.sec_mcycles);
                }
                else {
                    print!("sec has elapsed and reached max mcycle\n");
                }
                self.sec_mcycles = 0;
                self.current_time = Instant::now();
                PPUEvent::RenderEvent(RenderState::NoRender)
            }

        }
        // if self.sec_cycles < mcycle_per_sec && self.current_time.elapsed().as_secs() < one_sec {
        //     let cycles = self.cpu.tick(&mut self.mbc, &self.bios);
        //     self.sec_cycles += cycles;
        //     self.ppu.tick(&mut self.mbc, tile_frame, game_frame, cycles)
        // }



    }

}