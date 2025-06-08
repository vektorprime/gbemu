
use crate::gb::mbc::*;
use crate::gb::graphics::palette::*;
use crate::gb::graphics::tile::*;
use crate::gb::hwregisters::*;

pub struct Ppu {
   ly_inc_cycle: u64,
    // LCD and scrolling
    // pub lcdc: u8,  // FF40
    // pub stat: u8,  // FF41
    // pub scy: u8,   // FF42
    // pub scx: u8,   // FF43
    // pub ly: u8,    // FF44
    // pub lyc: u8,   // FF45
    // pub dma: u8,   // FF46
    // pub bgp: u8,   // FF47
    // pub obp0: u8,  // FF48
    // pub obp1: u8,  // FF49
    // pub wy: u8,    // FF4A
    // pub wx: u8,    // FF4B
    //
    // pub bit0_bg_win_priority_enable: bool,
    // pub bit1_obj_enable: bool, // 0 =  bg and win blank (white), and win display Bit is ignored in that case. Only objects may still be displayed (if enabled in Bit 1).
    // pub bit2_obj_size: bool, // 0 = 8×8; 1 = 8×16 obj sprites
    // pub bit3_bg_tile_map: bool, // 0 = 9800–9BFF; 1 = 9C00–9FFF
    // pub bit4_bg_win_tiles: bool, // 0 = 8800–97FF; 1 = 8000–8FFF
    // pub bit5_win_enable: bool,
    // pub bit6_win_tile_map: bool, // 0 = 9800–9BFF; 1 = 9C00–9FFF
    // pub bit7_lcd_ppu_enable: bool,
    pub tiles: Vec<Tile>, 
    //bg_tile_map: [u8; 1024],
}
impl Ppu {
    pub fn new() -> Self {
        Ppu {
            // bit0_bg_win_priority_enable: false,
            // bit1_obj_enable: false,
            // bit2_obj_size: false,
            // bit3_bg_tile_map: false,
            // bit4_bg_win_tiles: false,
            // bit5_win_enable: false,
            // bit6_win_tile_map: false,
            // bit7_lcd_ppu_enable: false,
            ly_inc_cycle: 0,
            // lcdc: 0,
            // stat: 0,
            // scy: 0,
            // scx: 0,
            // ly: 0,
            // lyc: 0,
            // dma: 0,
            // bgp: 0,
            // obp0: 0,
            // obp1: 0,
            // wy: 0,
            // wx: 0,

            tiles: Vec::new(), 
        }
    }
    
    pub fn debug_show_tiles(&self) {

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
            
        }

    }

    pub fn is_lcdc_bit3_bg_tile_map_set(&self, mbc: &Mbc) -> bool {
        if mbc.hw_reg.lcdc & 0b0000_1000 == 0b0000_1000 {
            true
        }
        else {
            false
        }
    }

    pub fn get_bg_tile_map(&self, mbc: &Mbc) -> [u8; 1024] {
        let mut bg_tile_map: [u8; 1024] = [0; 1024];
        let address = if self.is_lcdc_bit3_bg_tile_map_set(&mbc) {
            0x9800
        } else {
            0x9C00
        };

        for x in 0x0..0x3FF {
            bg_tile_map[x] = mbc.read(address); 
        }
        if mbc.read(address) != 0 {
            println!("Something inside of tile map");
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

    pub fn tick(&mut self, mbc: &mut Mbc, cycles: u64) {
        let trigger_ly_inc = 114;
        let max_ly_value = 153;
        self.ly_inc_cycle += cycles;
        if self.ly_inc_cycle >= trigger_ly_inc {
            mbc.hw_reg.ly += 1;
            if mbc.hw_reg.ly >= max_ly_value {
                mbc.hw_reg.ly = 0;
            }
            self.ly_inc_cycle = 0;
        }
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

