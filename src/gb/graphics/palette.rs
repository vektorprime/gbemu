


// palette is set via hardware register (mem location) 0xFF47, BG palette data aka BGP

#[derive(Copy, Clone)]
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
            4 => Self::Transparent,
            _ => Self::White,
        }
    }
    pub fn get_rgba_code(&self) -> [u8; 4] {
        match self {
            PaletteColor::White => [155, 188, 15, 0],
            PaletteColor::LightGray => [139, 172, 15, 0],
            PaletteColor::DarkGray => [48, 98, 48, 0],
            PaletteColor::Black => [15, 56, 15, 0],
            PaletteColor::Transparent => [0, 0, 0, 255],
        }
    }

}

pub struct BGPalette {
    id0: PaletteColor,
    id1: PaletteColor,
    id2: PaletteColor,
    id3: PaletteColor,
}

impl BGPalette {
    pub fn new() -> Self {
        BGPalette {
            id0: PaletteColor::White,
            id1: PaletteColor::White,
            id2: PaletteColor::White,
            id3: PaletteColor::White,
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
    pub fn new() -> Self {
        OBJPalette {
            // id0 is always transparent here so we ignore it
            id0: PaletteColor::Transparent,
            id1: PaletteColor::White,
            id2: PaletteColor::White,
            id3: PaletteColor::White,
        }
    }
}