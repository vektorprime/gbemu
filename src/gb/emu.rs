use std::hint;
use crate::gb::cpu::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::mbc::*;
use crate::gb::graphics::ppu::*;
use crate::gb::testcpu::*;
use crate::gb::joypad::Joypad;

use std::time::{Instant};
use std::sync::{Arc, Mutex};


pub struct Emu {
    pub cpu: Cpu,
    bios: Bios,
    pub mbc: Box<Mbc>, // mbc includes rom and ram
    pub ppu: Ppu,
    // pub lcd: Lcd,
    pub debug: bool,
    pub sec_mcycles: u64, // tracking max mcycles per sec
    pub frame_mcycles: u64,
    pub current_time: Instant,
    pub current_time_per_frame: Instant,
    pub is_cpu_test_enabled: bool,
    pub is_cpu_tested: bool,
    pub joypad: Arc<Mutex<Joypad>>
}

impl Emu {
    pub fn new(color_mode: ColorMode, joypad: Arc<Mutex<Joypad>>, debug: bool) -> Self {
        Emu {
            cpu: Cpu::new(),
            mbc: Box::new(Mbc::new()), // mbc has rom and ram
            bios: Bios::new(color_mode), 
            ppu: Ppu::new(),
            debug,
            sec_mcycles: 0, // tracking max mcycles per sec
            frame_mcycles: 0,
            current_time: Instant::now(),
            current_time_per_frame: Instant::now(),
            is_cpu_tested: false,
            is_cpu_test_enabled: true,
            joypad,
        }
    }

    pub fn load_rom_file(&mut self, file: String) {
        self.mbc.rom = Some(Rom::new(file.as_str()));
    }

    pub fn load_bios(&mut self) {
        self.mbc.boot_rom.load_bios_to_mem(&self.bios);
    }

    pub fn test_cpu() {
        println!("TESTING CPU");
        let all_cpu_tests = get_all_tests();
        let mut test_cpu = Cpu::new();
        let mut test_mbc =  Box::new(Mbc::new());
        test_mbc.is_testing_enabled = true;
        //println!("Got all tests, starting testing");
        for test in &all_cpu_tests {
            //println!("executing test {}", test.name);
            // setup state
                // registers
            setup_initial_registers(&mut test_cpu.registers, &test.initial_test_state);
            // ram
            for ram_entry in &test.initial_test_state.test_ram {
                test_mbc.write(ram_entry.0, ram_entry.1, OpSource::CPU);
            }
            // setup done

            // execute cpu
            test_cpu.tick(&mut test_mbc);


            // check state
                // registers
                let failed_registers = compare_registers(&test_cpu.registers, &test.final_test_state);
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
                test_mbc.write(ram_entry.0, 0, OpSource::CPU);
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
            Emu::test_cpu();
            self.is_cpu_tested = true;
        }

        //joypad
        {
            let mut joypad_unlocked = self.joypad.lock().unwrap();
            joypad_unlocked.sync_state(&mut self.mbc);
        }

        // //

        let mcycles_per_sec: u64 = 1_053_360;
        let mcycles_per_frame: u64 = 175_560;
        //let one_sec: u64 = 1;
        let elapsed_time = self.current_time.elapsed();
        //0.0166 f64 sec is 1/60 of a sec

        if elapsed_time.as_secs_f64() <= 1.0f64 {
            if self.sec_mcycles < mcycles_per_sec {
                if self.frame_mcycles < mcycles_per_frame {
                        let mcycles = self.cpu.tick(&mut self.mbc);
                        self.sec_mcycles += mcycles;
                        self.frame_mcycles += mcycles;
                        self.ppu.tick(&mut self.mbc, tw, bgmw, gw, mcycles)
                }
                else {
                    // hot spin the thread
                    while self.current_time_per_frame.elapsed().as_secs_f64() <= 0.166f64  {
                        //print!(" ");
                        hint::spin_loop();
                    }
                    self.current_time_per_frame = Instant::now();
                    self.frame_mcycles = 0;
                    return PPUEvent::RenderEvent(RenderState::NoRender);
                }

            } else {
                self.sec_mcycles = 0;
                self.current_time = Instant::now();
                return PPUEvent::RenderEvent(RenderState::NoRender);
            }

        }  else {
            if elapsed_time.as_secs_f64() > 1.0f64 && self.debug == false {
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