
use crate::gb::mbc::Mbc;



//used by fetcher functions
pub enum RequestedPalette {
    BG,
    Sprite,
}

// palette is set via hardware register (mem location) 0xFF47, BG palette data aka BGP

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum PaletteColor {
    White,
    LightGray,
    DarkGray,
    Black,
    Transparent,
}

impl PaletteColor {
    pub fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::White,
            1 => Self::LightGray,
            2 => Self::DarkGray,
            3 => Self::Black,
            //4 => Self::Transparent,
            _ => { panic!("Unknown palette color {}", n); },
        }
    }
    pub fn get_rgba_code(&self) -> [u8; 4] {
        match self {
            PaletteColor::White => [255, 255, 255, 255],
            PaletteColor::LightGray => [192, 192, 192, 255],
            PaletteColor::DarkGray => [96, 96, 96, 255],
            PaletteColor::Black => [0, 0, 0, 255],
            PaletteColor::Transparent => [255, 255, 255, 0],
        }
    }
}

pub enum Palette {
    OBJ(OBJPalette),
    BG(BGPalette),
}


pub struct BGPalette {
    id0: PaletteColor,
    id1: PaletteColor,
    id2: PaletteColor,
    id3: PaletteColor,
}

impl BGPalette {
    pub fn new(mbc: &Mbc) -> Self {
        BGPalette {
            id0: PaletteColor::from_u8(mbc.hw_reg.get_bgp_id0()),
            id1: PaletteColor::from_u8(mbc.hw_reg.get_bgp_id1()),
            id2: PaletteColor::from_u8(mbc.hw_reg.get_bgp_id2()),
            id3: PaletteColor::from_u8(mbc.hw_reg.get_bgp_id3()),
        }
    }
    pub fn from_u8(&self, n: u8) -> PaletteColor {
        match n {
            0 => self.id0,
            1 => self.id1,
            2 => self.id2,
            3 => self.id3,
            _ => {panic!("requested a bad PaletteColor in BGPalette::from_u8(n)")},
        }
    }
}


pub struct OBJPalette {
    id0: PaletteColor,
    id1: PaletteColor,
    id2: PaletteColor,
    id3: PaletteColor,
}

impl OBJPalette {
    pub fn new(mbc: &Mbc) -> Self {
       OBJPalette {
            id0: PaletteColor::Transparent,
            id1: PaletteColor::from_u8(mbc.hw_reg.get_obp0_id1()),
            id2: PaletteColor::from_u8(mbc.hw_reg.get_obp0_id2()),
            id3: PaletteColor::from_u8(mbc.hw_reg.get_obp0_id3()),
        }
    }
    pub fn from_u8(&self, n: u8) -> PaletteColor {
        match n {
            0 => self.id0,
            1 => self.id1,
            2 => self.id2,
            3 => self.id3,
            _ => {panic!("requested a bad PaletteColor in OBJPalette::from_u8(n)")},
        }
    }
}

