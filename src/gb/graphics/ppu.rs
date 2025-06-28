use crate::gb::emu::Emu;
use crate::gb::mbc::*;
use crate::gb::graphics::palette::*;
use crate::gb::graphics::tile::*;
use crate::gb::hwregisters::*;

#[derive(PartialEq)]
pub enum RenderState {
    Render,
    NoRender
}

pub struct Ppu {
    //ly_inc_cycle: u64,
    //frame_cycle: u64,
    tcycle_in_frame: u64,
    tcycle_in_scanline: u64,
    finished_mode_0_in_frame: bool,
    finished_mode_1_in_frame: bool,
    finished_mode_2_in_frame: bool,
    finished_mode_3_in_frame: bool,
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
    pub ppu_init_complete: bool,
    // pub active: bool,
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
            tcycle_in_frame: 0,
            tcycle_in_scanline: 0,
            finished_mode_0_in_frame: false,
            finished_mode_1_in_frame: false,
            finished_mode_2_in_frame: false,
            finished_mode_3_in_frame: false,
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
            ppu_init_complete: false,
            // active: false,
        }
    }
    
    pub fn debug_show_tiles(&self) {

    }


    pub fn load_all_tiles(&mut self, mbc: &Mbc) {
        self.tiles.clear();
        let address: u16 = 0x8000;

        // the whole tile range
        for x in (0..6144).step_by(16) {
            let mut new_tile = Tile::new();
            // every 16 bytes is a tile
            let mut temp_tile: [u8; 16] = [0; 16];
            for y in 0..16 {
                temp_tile[y] = mbc.read(address + x + (y as u16));
                if temp_tile[y] != 0 {
                    print!("Tile #{} byte {} is {:#x} \n", x / 16, y, temp_tile[y] );
                }
            }

            // decode every 2 bytes as a row
            // for (z, byte) in temp_tile.chunks_exact(2).enumerate() {
            //     new_tile.decode_tile_row(byte[0], byte[1], z);
            //     print!("decoding byte {:#x} and {:#x} in row {} \n", byte[0], byte[1], z);
            // }
            
            //for r in 0..8 {
                //new_tile.decode_tile_row(temp_tile[r], temp_tile[ r + 1], r);
                //print!("decoding byte {:#x} and {:#x} in row {} \n", temp_tile[0], temp_tile[1], r);
            //} 

            new_tile.decode_tile_row(temp_tile[0], temp_tile[1], 0);
            new_tile.decode_tile_row(temp_tile[2], temp_tile[3], 1);
            new_tile.decode_tile_row(temp_tile[4], temp_tile[5], 2);
            new_tile.decode_tile_row(temp_tile[6], temp_tile[7], 3);
            new_tile.decode_tile_row(temp_tile[8], temp_tile[9], 4);
            new_tile.decode_tile_row(temp_tile[10], temp_tile[11], 5);
            new_tile.decode_tile_row(temp_tile[12], temp_tile[13], 6);
            new_tile.decode_tile_row(temp_tile[14], temp_tile[15], 7);
                
            
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
            0x9C00
        } else {
            0x9800
        };

        for x in 0x0..0x3FF {
            bg_tile_map[x] = mbc.read(address); 
        }
        if mbc.read(address) != 0 {
            let pr = mbc.read(address);
            print!("Tile map entry is {} \n", pr);
        }
        bg_tile_map
    }

    pub fn mode_2_oam_scan(&mut self) {

        // search for obj that are in this scan line pos and add to vec?
        
    }


    pub fn draw_tiles_in_viewer(&self, tile_frame: &mut [u8], cycles: &u64) {
        if !self.ppu_init_complete { return; }
        let mut pixels_source: Vec<[u8; 4]> = Vec::new();
        // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile

        let mut tile_in_grid_count: usize = 0;
        let rows_per_grid: usize = 8;
        let mut tile_in_row_count = 0;
        let tiles_per_row: usize = 16;
        let rows_per_tile = 8;
        let pixels_per_row = 8;
        let num_of_pixels_to_pad: usize = 32;

        for row_of_tiles_in_grid in 0..rows_per_grid {
            for row in 0..rows_per_tile {
                tile_in_grid_count = 0;
                for tile in &self.tiles {
                    if row_of_tiles_in_grid > 0 {
                        if tile_in_grid_count < row_of_tiles_in_grid * tiles_per_row {
                            tile_in_grid_count += 1;
                            continue;
                        }
                    }
                    // pad some bytes because the tiles don't take the whole screen
                    if tile_in_row_count == tiles_per_row {
                        for i in 0..num_of_pixels_to_pad {
                            pixels_source.push([255, 255, 255, 255]);
                        }
                        tile_in_row_count = 0;
                        break;
                    } else {
                        // tile.data is an array of 8 arrays that each hold 8 PaletteColor
                        for pixel in 0..pixels_per_row {
                            //put each pixel into a vec so we can move it to the frame later
                            pixels_source.push(tile.data[row][pixel].get_rgba_code());
                        }
                        tile_in_row_count += 1;
                    }
                }
            }
        }
        // copy each pixel into the frame
        for (i, pixel) in tile_frame.chunks_exact_mut(4).enumerate() {
            if i < pixels_source.len() {
                //let test_slice = [0x00, 0x00, 0xFF, 0xFFu8];
                //pixel.copy_from_slice(&test_slice);
                pixel.copy_from_slice(&pixels_source[i]);
            }
        }
    }

    pub fn draw_tiles_in_game(&self, bg_map: [u8; 1024], game_frame: &mut [u8], cycles: &u64) {
        if !self.ppu_init_complete { return; }
        let mut pixels_source: Vec<[u8; 4]> = Vec::new();
        // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile
        
        let mut tile_in_grid_count: usize = 0;
        let rows_per_grid: usize = 32;
        let mut tile_in_row_count = 0;
        let tiles_per_row: usize = 32;
        let rows_per_tile = 8;
        let pixels_per_row = 8;
        let mut current_tile: usize = 0;
        for row_of_tiles_in_grid in 0..rows_per_grid {
            for row in 0..rows_per_tile {
                tile_in_grid_count = 0;
                let mut tile_index = bg_map[current_tile] as usize;
                if row_of_tiles_in_grid > 0 {
                    if tile_in_grid_count < row_of_tiles_in_grid * tiles_per_row {
                        tile_in_grid_count += 1;
                        continue;
                    }
                }
                // tile.data is an array of 8 arrays that each hold 8 PaletteColor
                for pixel in 0..pixels_per_row {
                    //put each pixel into a vec so we can move it to the frame later
                    pixels_source.push(self.tiles[tile_index].data[row][pixel].get_rgba_code());
                }
                tile_in_row_count += 1;
                current_tile += 1;

            }
        }
        // copy each pixel into the frame
        for (i, pixel) in game_frame.chunks_exact_mut(4).enumerate() {
            if i < pixels_source.len() {
                //let test_slice = [0x00, 0x00, 0xFF, 0xFFu8];
                //pixel.copy_from_slice(&test_slice);
                pixel.copy_from_slice(&pixels_source[i]);
            }
        }
    }

    pub fn mode_3_draw(&mut self, bg_map: [u8; 1024], tile_frame: &mut [u8], game_frame: &mut [u8], cycles: &u64) {

        self.draw_tiles_in_game(bg_map, game_frame, cycles);
    }

    pub fn mode_0_h_blank(&self) {
        
    }

    pub fn mode_1_v_blank(&mut self, mbc: &mut Mbc) {
        // end the drawing of pixels in the ppu

    }

    pub fn tick(&mut self, mbc: &mut Mbc, tile_frame: &mut [u8], game_frame: &mut [u8], cycles: u64) -> RenderState {

        let tcycle = cycles * 4;
        
        // don't tick ppu unless the lcdc says ppu is on
        // i went back and forth here but I left it on because it seems like it may work
        // the pc counter was inc slow but that was due to other reasons
        if !mbc.hw_reg.is_lcdc_bit7_enabled() {
           //print!("lcdc bit 7 not enabled yet, skipping ppu tick \n");
            return RenderState::NoRender;
        }

        if !self.ppu_init_complete {
            self.load_all_tiles(&mbc);
            self.ppu_init_complete = true;
            print!("ppu init complete \n");
        }

        if mbc.need_tile_update {
            self.load_all_tiles(&mbc);
            //print!("need tile_update \n");
            mbc.need_tile_update = false;
        }

        self.draw_tiles_in_viewer(tile_frame, &tcycle);

        //
        let bg_map = self.get_bg_tile_map(mbc);

        // go through all PPU modes
        // mode 2 + 3 + 0 stop after scan line 143
        self.tcycle_in_scanline += tcycle;
        self.tcycle_in_frame += tcycle;
        let mode_1_v_blank_first_scan_line = 144;
        let current_scanline = mbc.hw_reg.ly;
        //print!("current scan line is {}\n", current_scanline);
        //print!("current tcycle_in_scanline is {}\n", self.tcycle_in_scanline);
        if current_scanline < mode_1_v_blank_first_scan_line {
            // mode 2 is dot 0-80
            let mode_2_oam_scan_last_tcycle = 79;

            if self.tcycle_in_scanline < mode_2_oam_scan_last_tcycle && !self.finished_mode_2_in_frame {
                self.mode_2_oam_scan();
                print!("entering mode_2_oam_scan \n");
                // not updating tycle manually because I want the cpu and ppu in sync
                // self.tcycle_in_scanline = 79;
                self.finished_mode_2_in_frame = true;
            }
            
            let mode_3_drawing_first_tcycle = 80;
            if self.tcycle_in_scanline >= mode_3_drawing_first_tcycle  && !self.finished_mode_3_in_frame {
                // Mode 3 is between 172 and 289 dots, let's call it 172
                print!("entering mode_3_drawing \n");
                self.mode_3_draw(bg_map, tile_frame, game_frame, &cycles);
                self.finished_mode_3_in_frame = true;
            }

            let mode_0_h_blank_first_tcycle = 252;
            if self.tcycle_in_scanline >= mode_0_h_blank_first_tcycle && !self.finished_mode_0_in_frame {
                print!("entering mode_0_h_blank \n");
                // Mode 0 is the remainder of the dots left in the scan line (final dot is 456)
                self.mode_0_h_blank();
                self.finished_mode_0_in_frame = true;
            }
        } else {
            // last 10 scan lines are mode 1
            // 4560 dots or 10 scan lines (each scan line is 456 dots)
            let mode_1_v_blank_first_tcycle = 65664;
            if self.tcycle_in_frame >= mode_1_v_blank_first_tcycle && !self.finished_mode_1_in_frame {
                print!("entering mode_1_v_blank \n");
                self.mode_1_v_blank(mbc);
                self.finished_mode_1_in_frame = true;
            }
        }

        if self.finished_mode_2_in_frame && self.finished_mode_3_in_frame &&
                self.finished_mode_0_in_frame && self.finished_mode_1_in_frame {
            self.finished_mode_2_in_frame = false;
            self.finished_mode_3_in_frame = false;
            self.finished_mode_0_in_frame = false;
            self.finished_mode_1_in_frame = false;
        }

        // reset tcycle in scan line because max is 456
        // also inc LY
        if self.tcycle_in_scanline >= 456 {
            // this print is very freq
            //print!("tcycle_in_scanline >= 456, incrementing LY \n");
            self.tcycle_in_scanline = 0;
            mbc.hw_reg.ly += 1;
        }

        // max ly is 155
        let max_ly_value = 153;
        if mbc.hw_reg.ly >= max_ly_value {
            mbc.hw_reg.ly = 0;
            // this print freq is the same as the 1 sec pausing, that means the ppu and cpu are in sync
            //print!("ly hw reg is max, resetting to 0 \n");
        }

        let max_tcycle_in_frame = 70224;
        if self.tcycle_in_frame >= max_tcycle_in_frame {
            //print!("tcycle_in_frame is >= 70224, generating frame \n");
            self.tcycle_in_frame = 0;
            self.tcycle_in_scanline = 0;
            RenderState::Render
        } else {
            RenderState::NoRender
        }



        // let trigger_ly_inc = 114;
        // let max_ly_value = 153;
        // self.ly_inc_cycle += cycles;
        // if self.ly_inc_cycle >= trigger_ly_inc {
        //     mbc.hw_reg.ly += 1;
        //     //print!("incrementing ly hw reg to {} \n", mbc.hw_reg.ly);
        //     if mbc.hw_reg.ly >= max_ly_value {
        //         mbc.hw_reg.ly = 0;
        //         //print!("ly hw reg is max, resetting to 0 \n");
        //     }
        //     self.ly_inc_cycle = 0;
        // }
        //
        // let m_cycle_per_frame = 289 / 4; //4 dots per mcycle and there are 289 dots
        // let trigger_frame_count = m_cycle_per_frame;
        // self.frame_cycle += cycles;
        // if self.frame_cycle >= trigger_frame_count {
        //     //print!("triggering frame \n");
        //     self.frame_cycle = 0;

        //}
        //else {
        //    RenderState::NoRender
        //}
        
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

