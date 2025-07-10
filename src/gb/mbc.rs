
use crate::gb::ram::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::hwregisters::HardwareRegisters;

use std::thread::sleep;
use std::time::Duration;

pub const ROM_BANK_SIZE: u16 = 0x4000;
pub const RAM_BANK_SIZE: u16 = 0x4000;

pub struct Mbc {
    pub hw_reg: HardwareRegisters,
    pub ram: Ram,
    pub boot_rom: Ram,
    pub rom: Option<Rom>,
    rom_bank: u8,
    ram_bank: u8,
    wr_ram_bank: bool,
    pub rom_ram: RomRam,
    pub need_tile_update: bool,
    pub need_bg_map_update: bool,
    pub rom_bank_mode: RomBankMode,
    xram: Ram,
    vram: Ram,
    wram: Ram,
    oam: Ram,
    io: Ram,
    hram: Ram,
}

impl Mbc {

    pub fn new() -> Self {
        Mbc {
            hw_reg: HardwareRegisters::new(),
            ram: Ram::new(),
            boot_rom: Ram::new(),
            rom: None,
            rom_bank: 0,
            ram_bank: 0,
            wr_ram_bank: false,
            rom_ram: RomRam::new(),
            need_tile_update: false,
            need_bg_map_update: false,
            rom_bank_mode: RomBankMode::Simple,
            xram: Ram::new(),
            vram: Ram::new(),
            wram: Ram::new(),
            oam: Ram::new(),
            io: Ram::new(),
            hram: Ram::new(),
        }
    }

    pub fn load_rom_to_mem(&mut self) {
        // let rom_size = self.rom.as_ref().unwrap().rom_size;
        // let mut rom_banks = match rom_size {
        //     RomSize::KB_32  => 0,     // no bank
        //     RomSize::KB_64  => 4,     // 4 banks
        //     RomSize::KB_128 => 8,     // 8 banks
        //     RomSize::KB_256 => 16,    // 16 banks
        //     RomSize::KB_512 => 32,    // 32 banks
        //     RomSize::MB_1   => 64,    // 64 banks
        //     RomSize::MB_2   => 128,   // 128 banks
        //     RomSize::MB_4   => 256,   // 256 banks
        //     RomSize::MB_8   => 512,   // 512 banks
        //     _ => 0,
        // };

        for (mut i, byte) in self.rom.as_ref().unwrap().data.iter().copied().enumerate() {
            if  i >= self.ram.memory.len() {
                self.ram.memory.push(byte)
            } else {
                self.ram.memory[i] = byte;
            }
        }
    }

    pub fn get_tima_reg_tcycle_inc_count(&self) -> u64 {
        let clock_select = self.hw_reg.tima & 0b0000_0011;
        if clock_select == 0  {
            return 1024
        } else if clock_select == 1  {
            return 16
        } else if clock_select == 2  {
            return 64
        } else if clock_select == 3  {
            return 256
        }
        panic!("Unable to get tcycle count in get_tima_reg_tcycle_inc_count");
    }

    pub fn is_tac_bit2_enable_set(&self) -> bool {
        if self.hw_reg.tac & 0b0000_0100 == 0b0000_0100 {
            return true
        } else {
            return false
        }
    }

    pub fn read_rom(&self, address: u16) -> u8 {
        let rom_type = self.rom.as_ref().unwrap().get_rom_type();
        match rom_type {
            RomType::Rom_Only => {
                self.rom_only_read(address)
                //self.ram.read(address)
            },
            RomType::MBC1 => {
                //print!("read_rom MBC1\n");
                self.mbc1_read(address)
            },
            RomType::MBC3 => {
                self.mbc3_read(address)
            },
            _ => {
                panic!("unknown RomType in mbc.read()");
            }
        }
    }
    pub fn read_bios(&self, address: u16) -> u8 {
        //print!("address in read_bios is {:#x} \n", address);
        self.boot_rom.read(address)
    }

    pub fn read(&self, address: u16) -> u8 {
        // handle special reads: BIOS, hw reg, then ROM
        match address {
            // BIOS
            0x00..=0xFF=>  {
                if self.hw_reg.boot_rom_control == 0 {
                    //print!("b");
                    self.read_bios(address)
                }
                else {
                    //print!("r");
                    self.read_rom(address)
                }
            },
            // Joypad and serial
            0xFF00 => self.hw_reg.joyp,
            0xFF01 => self.hw_reg.sb,
            0xFF02 => self.hw_reg.sc,

            // Timer
            0xFF04 => self.hw_reg.div,
            0xFF05 => self.hw_reg.tima,
            0xFF06 => self.hw_reg.tma,
            0xFF07 => self.hw_reg.tac,

            // Interrupt flags
            0xFF0F => self.hw_reg.intflags,

            // LCD and scrolling
            0xFF40 => self.hw_reg.lcdc,
            0xFF41 => self.hw_reg.stat,
            0xFF42 => self.hw_reg.scy,
            0xFF43 => self.hw_reg.scx,
            0xFF44 => self.hw_reg.ly,
            0xFF45 => self.hw_reg.lyc,
            0xFF46 => {
                // print!("reading 0xFF46, DMA hw register \n");
                self.hw_reg.dma
            },
            0xFF47 => self.hw_reg.bgp,
            0xFF48 => self.hw_reg.obp0,
            0xFF49 => self.hw_reg.obp1,
            0xFF4A => self.hw_reg.wy,
            0xFF4B => self.hw_reg.wx,

            // Boot ROM control
            0xFF50 => {
                print!("reading 0xFF50, Boot ROM control hw register \n");
                self.hw_reg.boot_rom_control
            },

            // Audio (NR10–NR52)
            0xFF10 => self.hw_reg.nr10,
            0xFF11 => self.hw_reg.nr11,
            0xFF12 => self.hw_reg.nr12,
            0xFF13 => self.hw_reg.nr13,
            0xFF14 => self.hw_reg.nr14,
            0xFF16 => self.hw_reg.nr21,
            0xFF17 => self.hw_reg.nr22,
            0xFF18 => self.hw_reg.nr23,
            0xFF19 => self.hw_reg.nr24,
            0xFF1A => self.hw_reg.nr30,
            0xFF1B => self.hw_reg.nr31,
            0xFF1C => self.hw_reg.nr32,
            0xFF1D => self.hw_reg.nr33,
            0xFF1E => self.hw_reg.nr34,
            0xFF20 => self.hw_reg.nr41,
            0xFF21 => self.hw_reg.nr42,
            0xFF22 => self.hw_reg.nr43,
            0xFF23 => self.hw_reg.nr44,
            0xFF24 => self.hw_reg.nr50,
            0xFF25 => self.hw_reg.nr51,
            0xFF26 => self.hw_reg.nr52,

            // Wave pattern RAM: FF30–FF3F
            0xFF30..=0xFF3F => {
                let index = (address - 0xFF30) as usize;
                self.hw_reg.wave_pattern[index]
            }

            // Interrupt enable
            0xFFFF => self.hw_reg.ie,

            // Fallback to ROM or other areas
            _ => self.read_rom(address),
        }
    }

    pub fn write_rom(&mut self, address: u16, byte: u8) {
        if (0x8000..=0x97FF).contains(&address) {
            //print!("address in write_rom is {:#x} \n", address);
            self.need_tile_update = true;
        }
        if (0x9800..=0x9BFF).contains(&address) {
            self.need_bg_map_update = true;
            // if byte != 0x0 && byte != 0x2F {
            //     print!("address in write_rom is {:#x} and new value is {:#x} \n", address, byte);
            // }
            // if address == 0x9820 {
            //     print!("address in write_rom is {:#x} and new value is {:#x} \n", address, byte);
            //
            // }
            // if address == 0x9820 {
            //     if byte == 0x9B {
            //         std::thread::sleep(std::time::Duration::from_secs(10));
            //         print!("address in write_rom is {:#x} and new value is {:#x} \n", address, byte);
            //     }
            // }
            // if (0x9000..=0x9010).contains(&address) {
            //     print!("address in write_rom is {:#x} \n", address);
            //     self.need_tile_update = true;
            // }
        }

        let rom_type = self.rom.as_ref().unwrap().get_rom_type();
        match rom_type {
            RomType::Rom_Only => {
                //self.ram.write(address, byte);
                self.rom_only_write(address, byte);
            }
            RomType::MBC1 => {
                self.mbc1_write(address, byte);
            }
            RomType::MBC3 => {
                self.mbc3_write(address, byte);
            },
            _ => {
                panic!("unknown RomType in mbc.read()");
            }
        }
    }

    pub fn write_bios(&mut self, address: u16, byte: u8) {
        //print!("address in write_bios is {:#x} \n", address);
        self.boot_rom.write(address, byte);
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        // handle special writes: BIOS, hw reg, then ROM
        match address {
            // BIOS
            0x00..=0xFF=> {
                if self.hw_reg.boot_rom_control == 0 {
                   // self.write_bios(address, byte);
                    print!("writing while in bios to add {} byte {} \n", address, byte);
                }
                // else {
                //     //panic!("writing between 0x00 to 0xFF in write");
                //     self.write(address, byte);
                // }
            }


            // Joypad and serial
            0xFF00 => self.hw_reg.joyp = byte,
            // 0xFF01 => self.hw_reg.sb = byte,
            0xFF01 => {
                //print!("{}", byte as char);
                self.hw_reg.sb = byte;
            },
            0xFF02 =>  {
                if byte == 0x81 {
                    // If it is, immediately "complete" the transfer
                    print!("{}", self.hw_reg.sb as char); // Print the character
                    self.hw_reg.sc = 0x00; // Clear SC. This simulates the hardware clearing it after transfer
                } else {
                    // For other SC values, just store it
                    self.hw_reg.sc = byte;
                }
            }
            //self.hw_reg.sc = byte,

            // Timer
            //
            // DIV
            // appears as 8 bit to software that incrementd every tcycle
            // internally in hw it's 16 bit that increments every 256 tcycle
            // only upper 8 bits are mapped to mem
            // writing to FF04 resets it to 0
            // STOP inst also resets this and begins again after STOP ends
            0xFF04 => self.hw_reg.div = 0, // writing to DIV resets it
            0xFF05 => self.hw_reg.tima = byte,
            0xFF06 => self.hw_reg.tma = byte,
            0xFF07 => self.hw_reg.tac = byte,

            // Interrupt flags
            0xFF0F => self.hw_reg.intflags = byte,

            // LCD and scrolling
            0xFF40 => self.hw_reg.lcdc = byte,
            0xFF41 => self.hw_reg.stat = byte,
            0xFF42 => {
                //print!("writing {} to SCY\n", self.hw_reg.scy);
                self.hw_reg.scy = byte;
            },
            0xFF43 => {
                //print!("writing {} to SCX\n", self.hw_reg.scx);
                self.hw_reg.scx = byte;
            },
            0xFF44 => self.hw_reg.ly = 0, // writing to LY resets it
            0xFF45 => self.hw_reg.lyc = byte,
            0xFF46 => {
                if byte != 0 {
                    print!("writing to 0xFF46, DMA hw register \n");
                    let base_add = 0xFE00;
                    for i    in 0..160u16 {
                        let add= (byte as u16) << 8;
                        let val = self.read(add + i);
                        self.write(base_add + i, val);
                    }
                }
                self.hw_reg.dma = byte;
            },
            0xFF47 => self.hw_reg.bgp = byte,
            0xFF48 => self.hw_reg.obp0 = byte,
            0xFF49 => self.hw_reg.obp1 = byte,
            0xFF4A => {
                //print!("writing to WY\n");
                self.hw_reg.wy = byte;
            },
            0xFF4B => {
                //print!("writing to WX\n");
                self.hw_reg.wx = byte;
            },


            // Boot ROM control
            0xFF50 => {
                print!("writing {} to 0xFF50, Boot ROM control hw register \n", byte);
                self.hw_reg.boot_rom_control = byte
            },

            // Audio (NR10–NR52)
            0xFF10 => self.hw_reg.nr10 = byte,
            0xFF11 => self.hw_reg.nr11 = byte,
            0xFF12 => self.hw_reg.nr12 = byte,
            0xFF13 => self.hw_reg.nr13 = byte,
            0xFF14 => self.hw_reg.nr14 = byte,
            0xFF16 => self.hw_reg.nr21 = byte,
            0xFF17 => self.hw_reg.nr22 = byte,
            0xFF18 => self.hw_reg.nr23 = byte,
            0xFF19 => self.hw_reg.nr24 = byte,
            0xFF1A => self.hw_reg.nr30 = byte,
            0xFF1B => self.hw_reg.nr31 = byte,
            0xFF1C => self.hw_reg.nr32 = byte,
            0xFF1D => self.hw_reg.nr33 = byte,
            0xFF1E => self.hw_reg.nr34 = byte,
            0xFF20 => self.hw_reg.nr41 = byte,
            0xFF21 => self.hw_reg.nr42 = byte,
            0xFF22 => self.hw_reg.nr43 = byte,
            0xFF23 => self.hw_reg.nr44 = byte,
            0xFF24 => self.hw_reg.nr50 = byte,
            0xFF25 => self.hw_reg.nr51 = byte,
            0xFF26 => self.hw_reg.nr52 = byte,

            // Wave pattern RAM: FF30–FF3F
            0xFF30..=0xFF3F => {
                let index = (address - 0xFF30) as usize;
                self.hw_reg.wave_pattern[index] = byte;
            }

            // Interrupt enable
            0xFFFF => self.hw_reg.ie = byte,

            // Fallback to ROM or bios
            _ => self.write_rom(address, byte),
        }
    }
    pub fn rom_only_read(&self, address: u16) -> u8 {
        // read from rom bank 0
        if  (0x0000..=0x7FFF).contains(&address) {
            return self.rom.as_ref().unwrap().read(address as u32);
            //self.ram.read(address);
        } else if (0x8000..=0x9FFF).contains(&address) {
            // read V RAM
            let ram_offset: u16 = 0x8000;
            return self.vram.read(address - ram_offset);
        } else if (0xA000..=0xBFFF).contains(&address) {
            // read X RAM
            let ram_offset: u16 = 0xA000;
            return self.xram.read(address - ram_offset);
        } else if (0xC000..=0xDFFF).contains(&address) {
            // read W RAM
            let ram_offset: u16 = 0xC000;
            return self.wram.read(address - ram_offset);
        } else if (0xFE00..=0xFE9F).contains(&address) {
            // read OAM
            let oam_offset: u16 = 0xFE00;
            return self.oam.read(address - oam_offset);
        } else if (0xFF00..=0xFF7F).contains(&address) {
            // read IO
            let io_offset: u16 = 0xFF00;
            return self.io.read(address - io_offset);
        } else if (0xFF80..=0xFFFE).contains(&address) {
            // read HRAM
            let hram_offset: u16 = 0xFF80;
            return self.hram.read(address - hram_offset);
        }

        self.ram.read(address)
    }

    pub fn rom_only_write(&mut self, address: u16, byte: u8) {
        // read from rom bank 0
        if  (0x0000..=0x7FFF).contains(&address) {
            self.rom.as_mut().unwrap().write(address as u32, byte);
            //self.ram.write(address, byte);
            //panic!("write to rom in rom_only_write address {}, byte {}", address, byte);
        } else if (0x8000..=0x9FFF).contains(&address) {
            // read VRAM
            let ram_offset: u16 = 0x8000;
            self.vram.write(address - ram_offset, byte);
            return;
        } else if (0xA000..=0xBFFF).contains(&address) {
            // read XRAM
            let ram_offset: u16 = 0xA000;
            self.xram.write(address - ram_offset, byte);
            return;
        } else if (0xC000..=0xDFFF).contains(&address) {
            // read WRAM
            let ram_offset: u16 = 0xC000;
            self.wram.write(address - ram_offset, byte);
            return;
        } else if (0xFE00..=0xFE9F).contains(&address) {
            // read OAM
            let oam_offset: u16 = 0xFE00;
            return self.oam.write(address - oam_offset, byte);
        } else if (0xFF00..=0xFF7F).contains(&address) {
            // read IO
            let io_offset: u16 = 0xFF00;
            return self.io.write(address - io_offset, byte);
        } else if (0xFF80..=0xFFFE).contains(&address) {
            // read HRAM
            let hram_offset: u16 = 0xFF80;
            return self.hram.write(address - hram_offset, byte);
        }

        // print!("writing to rom in rom_only_write address: {}, byte: {}\n", address, byte);
        self.ram.write(address, byte);
    }
    pub fn mbc1_read(&self, address: u16) -> u8 {
        let current_bank_mode = self.rom_bank_mode;
        // read from rom bank 0
        if  (0x0000..=0x3FFF).contains(&address) {
            //if current_bank_mode == RomBankMode::Simple {
                return self.rom.as_ref().unwrap().read(address as u32);
            // } else {
            //    if self.rom_bank > 0x10 {
            //        // eg add is 0x0010 and bank is 0x17
            //        // 0x17 * 0x4000 + 0x10 = 0x5C010
            //        let offset: u32 = (address as u32);
            //        let rom_bank_add: u32 = (self.rom_bank as u32) * 0x4000;
            //        let calculated_address:u32 = rom_bank_add + offset;
            //        return self.rom.as_ref().unwrap().read(calculated_address);
            //    }
            // }
        }  else if (0x4000..=0x7FFF).contains(&address) {
            // read from rom bank 1 to X
            if self.rom_bank == 1 {
                return self.rom.as_ref().unwrap().read((address as u32));
            } else {
                // eg add is 0x0010 and bank is 0x17
                // 0x17 * 0x4000 + 0x10 = 0x5C010
                let offset: u32 = (address as u32) - 0x4000;
                let rom_bank_add: u32 = (self.rom_bank as u32) * 0x4000;
                let calculated_add: u32 = rom_bank_add + offset;
                return self.rom.as_ref().unwrap().read(calculated_add);
            }
        } else if (0x8000..=0x9FFF).contains(&address) {
            // read VRAM
            let vram_base_size: u16 = 0x8000;
            return self.vram.read(address - vram_base_size);
        } else if (0xA000..=0xBFFF).contains(&address) {
            //read XRAM
            let ram_bank_size: u16 = 0x2000;
            let ram_base_size: u16 = 0xA000;
            // read from ram bank 0 to 3
            if self.ram_bank == 0 {
                return self.xram.read(address - ram_base_size);
            } else {
                let offset  = address - ram_base_size;
                let ram_bank_add = ram_bank_size * (self.ram_bank as u16);
                let calculated_add= ram_bank_add + offset;
                return self.xram.read(calculated_add);
            }
        } else if (0xC000..=0xDFFF).contains(&address) {
            // read WRAM
            let ram_base_size: u16 = 0xC000;
            return self.wram.read(address - ram_base_size);
        } else if (0xFE00..=0xFE9F).contains(&address) {
            // read OAM
            let oam_offset: u16 = 0xFE00;
            return self.oam.read(address - oam_offset);
        } else if (0xFF00..=0xFF7F).contains(&address) {
            // read IO
            let io_offset: u16 = 0xFF00;
            return self.io.read(address - io_offset);
        } else if (0xFF80..=0xFFFE).contains(&address) {
            // read HRAM
            let hram_offset: u16 = 0xFF80;
            return self.hram.read(address - hram_offset);
        }

        //print!("Unhandled read in mbc1_read at add {}\n", address);
        self.ram.read(address)
    }

    pub fn mbc1_write(&mut self, address: u16, byte: u8) {
        let current_rom_size = self.rom.as_ref().unwrap().rom_size;
        let current_ram_size = self.rom.as_ref().unwrap().ram_size;
        // enable or diable ram write
        if (0x0000..=0x1FFF).contains(&address) {
            if (byte & 0x0F) == 0x0A {
                self.wr_ram_bank = true;
            }
            else {
                self.wr_ram_bank = false;
            }
            return;
        }
        // switch rom banks
        else if (0x2000..=0x3FFF).contains(&address) {
            // sets the rom bank number for 0x4000-7FFF
            let bank = byte & 0b0001_1111;
            if bank == 0 {
                self.rom_bank = 1;
            } else {
                self.rom_bank = bank;

            }
            return;
            // for 1MB roms - bit 5-6 of ROM bank num
            // for 32KB ram - RAM bank num
            // else ignore
        } else if  (0x4000..=0x5FFF).contains(&address) {
            let value = byte & 0b_0000_0011; // Only the lower 2 bits matter

            if self.rom_bank_mode == RomBankMode::Advanced {
                // Use bits 5-6 for upper ROM bank bits
                let upper_bits = value << 5;
                self.rom_bank = (self.rom_bank & 0b0001_1111) | upper_bits;
            } else {
                // Use the value as RAM bank number (if applicable)
                self.ram_bank = value;
            }
            return;
        }  else if (0x6000..=0x7FFF).contains(&address) {
            // write the banking mode
            // simple 0000-3FFF and A000-BFFF are bank 0 of ROM and SRAM
            // advanced 0000-3FFF and A000-BFFF can be bank switched via 4000-5FFF hw reg
            if byte == 0 {
                self.rom_bank_mode = RomBankMode::Simple;
            }
            else {
                self.rom_bank_mode = RomBankMode::Advanced;
            }
            return;
        } else if (0x8000..=0x9FFF).contains(&address) {
            // write to VRAM
            let vram_base_size: u16 = 0x8000;
            self.vram.write(address - vram_base_size, byte);
            return;
        } else if (0xA000..=0xBFFF).contains(&address) {
            //write to XRAM
            let ram_bank_size: u16 = 0x2000;
            let ram_base_size: u16 = 0xA000;
            // read from ram bank 0 to 3
            if self.ram_bank == 0 {
                self.xram.write(address - ram_base_size, byte);
                return;
            } else {
                let offset  = address - ram_base_size;
                let ram_bank_add = ram_bank_size * (self.ram_bank as u16);
                let calculated_add= ram_bank_add + offset;
                self.xram.write(calculated_add, byte);
                return;
            }
        } else if (0xC000..=0xDFFF).contains(&address) {
            // read WRAM
            let ram_base_size: u16 = 0xC000;
            self.wram.write(address - ram_base_size, byte);
            return;
        } else if (0xFE00..=0xFE9F).contains(&address) {
            // read OAM
            let oam_offset: u16 = 0xFE00;
            self.oam.write(address - oam_offset, byte);
            return;
        } else if (0xFF00..=0xFF7F).contains(&address) {
            // read IO
            let io_offset: u16 = 0xFF00;
            self.io.write(address - io_offset, byte);
            return;
        } else if (0xFF80..=0xFFFE).contains(&address) {
            // read HRAM
            let hram_offset: u16 = 0xFF80;
            self.hram.write(address - hram_offset, byte);
            return;
        }

        //print!("attempted an unhandled write inside mbc1_write address {} byte {:X}\n", address, byte);
        self.ram.write(address, byte);
        //panic!("attempted an unhandled write inside mbc1_write address {} byte {:X}", address, byte);
    }

    pub fn mbc3_read(&self, address: u16) -> u8 {
        // read from rom bank 0
        if  (0x0000..=0x3FFF).contains(&address) {
            self.rom.as_ref().unwrap().read(address as u32);
        } else if (0x4000..=0x7FFF).contains(&address) { // read from rom bank 1 to X
            let base: u32 = (self.rom_bank as u32) * 0x4000;
            let offset: u32 = (address as u32) - 0x4000;
            let calculated_address:u32 = base + offset;
            return self.rom.as_ref().unwrap().read(calculated_address);
        }

        self.rom.as_ref().unwrap().read(address as u32)
    }

    pub fn mbc3_write(&mut self, address: u16, byte: u8) -> u8 {
        // enable or diable ram write
        if (0x0000..=0x1FFF).contains(&address) {
            if byte & 0xA0 == 0xA0 {
                self.wr_ram_bank = true;
            }
            else {
                self.wr_ram_bank = false;
            }
        }
        // switch rom banks
        else if (0x2000..=0x3FFF).contains(&address) {
            // sets the rom bank number for 0x4000-7FFF
            if byte == 0 {
                // 0 must always be rom bank 1
                self.rom_bank = 1;
            }
            else {
                // this is a 5 bit value so discard any higher bits
                self.rom_bank = byte & 0b_0001_1111;
            }
        }
        // for 1MB roms - bit 5-6 of ROM bank num
        // for 32KB ram - RAM bank num
        // else ignore

        else if  (0x4000..=0x5FFF).contains(&address) {
            if self.rom.as_ref().unwrap().rom_size == RomSize::MB_1 {

            }
        }
        // write to ram bank X
        else if (0xA000..=0xBFFF).contains(&address) {
            let base = (self.ram_bank as u32) * 0x2000;
            let offset = (address as u32) - 0xA000;
            let calculated_address = base + offset;
            self.rom_ram.write(calculated_address, byte);
        }
        panic!("attempted an unhandled write inside mbc3_write address {} byte {:X}", address, byte);
    }
}