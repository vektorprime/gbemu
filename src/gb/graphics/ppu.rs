
use crate::gb::mbc::*;
use crate::gb::graphics::palette::*;
use crate::gb::graphics::tile::*;
pub struct Ppu {
    bit0_bg_win_priority_enable: bool,
    bit1_obj_enable: bool, // 0 =  bg and win blank (white), and win display Bit is ignored in that case. Only objects may still be displayed (if enabled in Bit 1).
    bit2_obj_size: bool, // 0 = 8×8; 1 = 8×16 obj sprites
    bit3_bg_tile_map: bool, // 0 = 9800–9BFF; 1 = 9C00–9FFF
    bit4_bg_win_tiles: bool, // 0 = 8800–97FF; 1 = 8000–8FFF
    bit5_win_enable: bool,
    bit6_win_tile_map: bool, // 0 = 9800–9BFF; 1 = 9C00–9FFF
    bit7_lcd_ppu_enable: bool, 
    tiles: Vec<Tile>, 
    //bg_tile_map: [u8; 1024],
}
impl Ppu {
    pub fn new() -> Self {
        Ppu {
            bit0_bg_win_priority_enable: false,
            bit1_obj_enable: false, 
            bit2_obj_size: false, 
            bit3_bg_tile_map: false, 
            bit4_bg_win_tiles: false, 
            bit5_win_enable: false,
            bit6_win_tile_map: false, 
            bit7_lcd_ppu_enable: false, 
            tiles: Vec::new(), 
        }
    }
    


    pub fn load_all_tiles(&mut self, mbc: &Mbc) {
        
        let address: u16 = 0x8000;

        // the whole tile range
        for x in (0..6144).step_by(16) {
            let mut new_tile = Tile::new();
            // every 16 bytes is a tile
            let mut temp_tile: [u8; 16] = [0; 16];
            for y in 0..16 {
                temp_tile[y] = mbc.read(address + x + (y as u16));
            } 

            // decode every 2 bytes as a row
            for (z, byte) in temp_tile.chunks_exact(2).enumerate() {
                new_tile.decode_tile_row(byte[0], byte[1], z);
            }

            // store in self.tiles vec
            self.tiles.push(new_tile);

            // clear temp_tile
            temp_tile = [0; 16];
            
        }


    }

    pub fn get_bg_tile_map(&self, mbc: &Mbc) -> [u8; 1024] {
        let mut bg_tile_map: [u8; 1024] = [0; 1024];
        let address = if self.bit3_bg_tile_map {
            0x9800
        } else {
            0x9C00
        };

        for x in 0x0..0x3FF {
            bg_tile_map[x] = mbc.read(address); 
        }

        bg_tile_map
    }

    pub fn mode_2_oam_scan(&self) {
        // search for obj that are in this scan line pos and add to vec?

    }


    pub fn mode_3_draw(&self) {
        // draw bg
        // 32 x 32 tiles
    }

    pub fn mode_0_h_blank(&self) {
        
    }

    pub fn mode_1_v_blank(&self) {
        
    }

    pub fn tick(&self, mbc: &Mbc) {

        let bg_map = self.get_bg_tile_map(mbc);
        self.mode_2_oam_scan();
        self.mode_3_draw();
        self.mode_0_h_blank();

    }

}


#[derive(Default)]
pub struct Oam {
    byte0_y_pos: u8,
    byte1_x_pos: u8,
    byte2_tile_num: u8,
    byte3_sprite_flags: u8,
}

impl Oam {
    pub fn get_byte3_bit_4(&self) -> bool {
        if self.byte3_sprite_flags & 0b0001_0000 == 0b0001_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_4(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0001_0000;
    }

    pub fn get_byte3_bit_5(&self) -> bool {
        if self.byte3_sprite_flags & 0b0010_0000 == 0b0010_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_5(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0010_0000;
    }

    pub fn get_byte3_bit_6(&self) -> bool {
        if self.byte3_sprite_flags & 0b0100_0000 == 0b0100_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_6(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0100_0000;
    }

    pub fn get_byte3_bit_7(&self) -> bool {
        if self.byte3_sprite_flags & 0b1000_0000 == 0b1000_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_7(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b1000_0000;
    }
}

