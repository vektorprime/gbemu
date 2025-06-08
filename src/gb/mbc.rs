
use crate::gb::ram::*;
use crate::gb::rom::*;
use crate::gb::bios::*;
use crate::gb::hwregisters::HardwareRegisters;

pub const ROM_BANK_SIZE: u16 = 0x4000;
pub const RAM_BANK_SIZE: u16 = 0x4000;

pub struct Mbc {
    pub hw_reg: HardwareRegisters,
    pub ram: Ram,
    pub rom: Option<Rom>,
    rom_bank: u8,
    ram_bank: u8,
    wr_ram_bank: bool,
    pub rom_ram: RomRam,
}

impl Mbc {

    pub fn new() -> Self {
        Mbc {
            hw_reg: HardwareRegisters::new(),
            ram: Ram::new(),
            rom: None,
            rom_bank: 0,
            ram_bank: 0,
            wr_ram_bank: false,
            rom_ram: RomRam::new(),
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
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF40 => {
                return self.hw_reg.lcdc;
            },
            0xFF41 => {
                return self.hw_reg.stat;
            },
            0xFF42 => {
                return self.hw_reg.scy;
            },
            0xFF43 => {
                return self.hw_reg.scx;
            },
            0xFF44 => {
                return self.hw_reg.ly;
            },
            0xFF45 => {
                return self.hw_reg.lyc;
            },
            0xFF46 => {
                return self.hw_reg.dma;
            },
            0xFF47 => {
                return self.hw_reg.bgp;
            },
            0xFF48 => {
                return self.hw_reg.obp0;
            },
            0xFF49 => {
                return self.hw_reg.obp1;
            },
            0xFF4A => {
                return self.hw_reg.wy;
            },
            0xFF4B => {
                return self.hw_reg.wx;
            },
            _ => {
                return self.read_rom(address);
            }
        }
    }

    pub fn write_rom(&mut self, address: u16, byte: u8) {
        let rom_type = self.rom.as_ref().unwrap().get_rom_type();
        
        match rom_type {
            RomType::Rom_Only => {
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

    pub fn write(&mut self, address: u16, byte: u8) {
        match address {
            0xFF40 => {
                self.hw_reg.lcdc = byte;
            },
            0xFF41 => {
                self.hw_reg.stat = byte;
            },
            0xFF42 => {
                self.hw_reg.scy = byte;
            },
            0xFF43 => {
                self.hw_reg.scx = byte;
            },
            0xFF44 => {
                self.hw_reg.ly = byte;
            },
            0xFF45 => {
                self.hw_reg.lyc = byte;
            },
            0xFF46 => {
                self.hw_reg.dma = byte;
            },
            0xFF47 => {
                self.hw_reg.bgp = byte;
            },
            0xFF48 => {
                self.hw_reg.obp0 = byte;
            },
            0xFF49 => {
                self.hw_reg.obp1 = byte;
            },
            0xFF4A => {
                self.hw_reg.wy = byte;
            },
            0xFF4B => {
                self.hw_reg.wx = byte;
            },
            _ => {
                self.write_rom(address, byte);
            }
        }
    }
}