
use crate::gb::graphics::palette::*;


pub struct Tile {
    pub data: [[PaletteColor; 8]; 8],
}

impl Tile {
    pub fn new() -> Self {
        Tile {
            data: [[PaletteColor::Black; 8]; 8]
        }
    }
    
    pub fn decode_tile_row(&mut self, byte1: u8, byte2: u8, row: usize) {

        let b0: u8 = 0b0000_0001;
        let b1: u8 = 0b0000_0010;
        let b2: u8 = 0b0000_0100;
        let b3: u8 = 0b0000_1000;
        let b4: u8 = 0b0001_0000;
        let b5: u8 = 0b0010_0000;
        let b6: u8 = 0b0100_0000;
        let b7: u8 = 0b1000_0000;

        let pixel0: PaletteColor = {
            let mut p1: u8 = byte2 & b7;
            let mut p2: u8 = byte1 & b7;
            p1 >>= 7;
            p2 >>= 7; 
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        let pixel1 = {
            let mut p1: u8 = byte2 & b6;
            let mut p2: u8 = byte1 & b6;
            p1 >>= 6;
            p2 >>= 6;
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        let pixel2 = {
            let mut p1: u8 = byte2 & b5;
            let mut p2: u8 = byte1 & b5;
            p1 >>= 5;
            p2 >>= 5;
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        let pixel3 = {
            let mut p1: u8 = byte2 & b4;
            let mut p2: u8 = byte1 & b4;
            p1 >>= 4;
            p2 >>= 4;
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        let pixel4 = {
            let mut p1: u8 = byte2 & b3;
            let mut p2: u8 = byte1 & b3;
            p1 >>= 3;
            p2 >>= 3;
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        let pixel5 = {
            let mut p1: u8 = byte2 & b2;
            let mut p2: u8 = byte1 & b2;
            p1 >>= 2;
            p2 >>= 2;
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        let pixel6 = {
            let mut p1: u8 = byte2 & b1;
            let mut p2: u8 = byte1 & b1;
            p1 >>= 1;
            p2 >>= 1;
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        let pixel7 = {
            let mut p1: u8 = byte2 & b0;
            let p2: u8 = byte1 & b0;
            p1 <<= 1;
            PaletteColor::from_u8(p1 | p2)
        };

        self.data[row][0] = pixel0;
        self.data[row][1] = pixel1;
        self.data[row][2] = pixel2;
        self.data[row][3] = pixel3;
        self.data[row][4] = pixel4;
        self.data[row][5] = pixel5;
        self.data[row][6] = pixel6;
        self.data[row][7] = pixel7;

    }

}


