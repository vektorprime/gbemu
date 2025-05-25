use std::fs;
use std::io::ErrorKind;
use std::io::Write;


use crate::gb::mbc::*;

// need to dynamically load the banks based on the rom
// 

#[derive(Debug, Copy, Clone)]
pub enum RomType {
    None,
    Rom_Only,
    MBC1,
    MBC1_RAM,
    MBC1_RAM_BATT,
    MBC2,
    MBC2_BATT,
    MBC3,
    MBC3_RAM,
    MBC3_RAM_BATT,
    MBC3_RAM_BATT_RTC,
    MBC3_BATT_RTC,
    MBC5,
    MBC5_RAM,
    MBC5_RAM_BATT,
    MBC5_RAM_BATT_RTC,
    MBC5_BATT_RTC,
    // https://gbdk.org/docs/api/docs_rombanking_mbcs.html
}

#[derive(Debug, Copy, Clone)]
pub enum RomSize {
    Zero,
    KB_32,
    KB_64,
    KB_128,
    KB_256,
    KB_512,
    MB_1,
    MB_2,
    MB_4,
    MB_8,
    MB_16,
}

#[derive(Debug, Copy, Clone)]
pub enum RamSize {
    Zero,
    KB_2,
    KB_8,
    KB_32,
    KB_64,
    KB_128,
} 

pub struct Rom {
    pub data: Vec<u8>,
    pub rom_type: RomType,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
    // pub ram: Vec<Vec<u8>>,
}


impl Rom {
    pub fn new(file: &str) -> Self {
        let mut rom = Rom {
            data: fs::read(file).unwrap(),
            rom_type: RomType::None,
            rom_size: RomSize::Zero,
            ram_size: RamSize::Zero,
        };

        let rom_type = match rom.data[0x147] {
            0x00 => RomType::Rom_Only,          // max rom 32KB
            0x01 => RomType::MBC1,              // max rom 2MB
            0x02 => RomType::MBC1_RAM,          // max rom 2MB
            0x03 => RomType::MBC1_RAM_BATT,     // max rom 2MB
            0x05 => RomType::MBC2,              // max rom 256KB
            0x06 => RomType::MBC2_BATT,         // max rom 256KB
            0x0F => RomType::MBC3_BATT_RTC,     // max rom 2MB
            0x10 => RomType::MBC3_RAM_BATT_RTC, // max rom 2MB
            0x11 => RomType::MBC3,              // max rom 2MB
            0x12 => RomType::MBC3_RAM,          // max rom 2MB
            0x13 => RomType::MBC3_RAM_BATT,     // max rom 2MB
            0x19 => RomType::MBC5,              // max rom 8MB
            0x1A => RomType::MBC5_RAM,          // max rom 8MB
            0x1B => RomType::MBC5_RAM_BATT,     // max rom 8MB
            _    => RomType::Rom_Only, 
            // some missing types
        }; 

        let rom_size = match rom.data[0x148] {
            0x00 => RomSize::KB_32,     //   no bank
            0x01 => RomSize::KB_64,     //   4 banks
            0x02 => RomSize::KB_128,    //   8 banks
            0x03 => RomSize::KB_256,    //  16 banks
            0x04 => RomSize::KB_512,    //  32 banks
            0x05 => RomSize::MB_1,      //  64 banks
            0x06 => RomSize::MB_2,      // 128 banks
            0x07 => RomSize::MB_4,      // 256 banks
            0x08 => RomSize::MB_8,      // 512 banks
            _    => RomSize::Zero,
            //some missing banks
        };

        let ram_size = match rom.data[0x149] {
            0x00 => RamSize::Zero,
            0x01 => RamSize::KB_2,      // unused
            0x02 => RamSize::KB_8,      // 1  bank
            0x03 => RamSize::KB_32,     // 4  banks
            0x04 => RamSize::KB_128,    // 16 banks
            0x05 => RamSize::KB_64,     // 8  banks
            _    => RamSize::Zero, 
        };

        rom.rom_type = rom_type;
        rom.rom_size = rom_size;
        rom.ram_size = ram_size;
        rom

    }

    pub fn get_rom_type(&self) -> RomType {
        self.rom_type
    }

    pub fn read(&self, address: u32) -> u8 {
        self.data[address as usize]
    }
}