
use crate::gb::ram::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::hwregisters::HardwareRegisters;

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
        }
    }

    pub fn load_rom_to_mem(&mut self) {
        let rom_size = self.rom.as_ref().unwrap().rom_size;
        let mut rom_banks = match rom_size {
            RomSize::KB_32  => 0,    //   no bank
            RomSize::KB_64  => 4,  //   4 banks
            RomSize::KB_128 => 8,   //   8 banks
            RomSize::KB_256 => 16,   //  16 banks
            RomSize::KB_512 => 32,   //  32 banks
            RomSize::MB_1   => 64,   //  64 banks
            RomSize::MB_2   => 128,   // 128 banks
            RomSize::MB_4   => 256,   // 256 banks
            RomSize::MB_8   => 512,   // 512 banks
            _ => 0,
        };

        for (mut i, byte) in self.rom.as_ref().unwrap().data.iter().copied().enumerate() {
            // skip rom banks and copy only what's needed
            if i == 0x4000 && rom_banks > 0 {
                i = 0x4000 * rom_banks;
            }
            self.ram.memory[i] = byte;
        }
    }

    pub fn mbc3_read(&self, address: u16) -> u8 {  
        // read from rom bank 0
        if  (0x0000..0x3FFF).contains(&address) {
            self.rom.as_ref().unwrap().read(address as u32);
        }
        // read from rom bank 1 to X
        else if (0x4000..0x7FFF).contains(&address) {
            let base: u32 = (self.rom_bank as u32) * 0x4000;
            let offset: u32 = (address as u32) - 0x4000;
            let calculated_address:u32 = base + offset;
            return self.rom.as_ref().unwrap().read(calculated_address);
        }  

        self.rom.as_ref().unwrap().read(address as u32)
    }

    pub fn mbc3_write(&mut self, address: u16, byte: u8) -> u8 {  
        // enable or diable ram write
        if (0x0000..0x1FFF).contains(&address) {
            if byte == 0 {
                self.wr_ram_bank = false;
            }
            else {
                self.wr_ram_bank = true;
            }
        }   
        // switch rom banks
        else if (0x2000..0x3FFF).contains(&address) {
            if byte == 0 {
                // 0 must always be 1
                self.rom_bank = 1;
            }
            else {
                self.rom_bank = byte;
            }
        }
        // switch ram banks
        else if  (0x4000..0x5FFF).contains(&address) {
            self.ram_bank = byte;
        }
        // write to ram bank X
        else if (0xA000..0xBFFF).contains(&address) {
            let base = (self.ram_bank as u32) * 0x2000;
            let offset = (address as u32) - 0xA000;
            let calculated_address = base + offset;
            self.rom_ram.write(calculated_address, byte);
        }
        panic!("attempted an unhandled write inside mbc3_write");
    }
    pub fn read_rom(&self, address: u16) -> u8 {
        let rom_type = self.rom.as_ref().unwrap().get_rom_type();
        match rom_type {
            RomType::Rom_Only => {
                return self.ram.read(address);
            },
            RomType::MBC3 => {
                return self.mbc3_read(address);
            },
            _ => {
                panic!("unknown RomType in mbc.read()");
            }
        }
    }
    pub fn read_bios(&self, address: u16) -> u8 {
        //print!("address in read_bios is {:#x} \n", address);
        return self.boot_rom.read(address);
    }

    pub fn read(&self, address: u16) -> u8 {
        // handle special reads: BIOS, hw reg, then ROM
        match address {
            // BIOS
            0x00..=0xFF=>  {
                if self.hw_reg.boot_rom_control == 0 {
                    self.read_bios(address)
                }
                else {
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
            0xFF46 => self.hw_reg.dma,
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
        let rom_type = self.rom.as_ref().unwrap().get_rom_type();
        
        match rom_type {
            RomType::Rom_Only => {
                if (0x8000..=0x97FF).contains(&address) {
                    print!("address in write_rom is {:#x} \n", address);
                    self.need_tile_update = true;
                }
                // if (0x9000..=0x9010).contains(&address) {
                //     print!("address in write_rom is {:#x} \n", address);
                //     self.need_tile_update = true;
                // }
                self.ram.write(address, byte);
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
                    self.write_bios(address, byte);
                }
                else {
                    self.write(address, byte);
                }
            }


            // Joypad and serial
            0xFF00 => self.hw_reg.joyp = byte,
            0xFF01 => self.hw_reg.sb = byte,
            0xFF02 => self.hw_reg.sc = byte,

            // Timer
            0xFF04 => self.hw_reg.div = 0, // writing to DIV resets it
            0xFF05 => self.hw_reg.tima = byte,
            0xFF06 => self.hw_reg.tma = byte,
            0xFF07 => self.hw_reg.tac = byte,

            // Interrupt flags
            0xFF0F => self.hw_reg.intflags = byte,

            // LCD and scrolling
            0xFF40 => self.hw_reg.lcdc = byte,
            0xFF41 => self.hw_reg.stat = byte,
            0xFF42 => self.hw_reg.scy = byte,
            0xFF43 => self.hw_reg.scx = byte,
            0xFF44 => self.hw_reg.ly = 0, // writing to LY resets it
            0xFF45 => self.hw_reg.lyc = byte,
            0xFF46 => self.hw_reg.dma = byte,
            0xFF47 => self.hw_reg.bgp = byte,
            0xFF48 => self.hw_reg.obp0 = byte,
            0xFF49 => self.hw_reg.obp1 = byte,
            0xFF4A => self.hw_reg.wy = byte,
            0xFF4B => self.hw_reg.wx = byte,

            // Boot ROM control
            0xFF50 => {
                print!("writing to 0xFF50, Boot ROM control hw register \n");
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
}