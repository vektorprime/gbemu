


// palette is set via hardware register (mem location) 0xFF47, BG palette data aka BGP

pub enum PaletteColor {
    White,
    LightGray,
    DarkGray,
    Black,
    Transparent,
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