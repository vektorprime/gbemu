use crate::gb::emu::Emu;
use crate::gb::mbc::*;
use crate::gb::graphics::palette::*;
use crate::gb::graphics::tile::*;
use crate::gb::hwregisters::*;

use crate::gb::gbwindow::*;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use pixels::Pixels;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Interrupt {
    Stat_48,
    Timer_50,
    Serial_58,
    Joypad_60,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PPUEvent {
    RenderEvent(RenderState),
    InterruptEvent(Interrupt),
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderState {
    Render,
    NoRender
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PPUMode {
    H_Blank,
    V_Blank,
    OAM_Scan,
    Draw,
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
    pub bg_tile_map: [u8; 1024],
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
            bg_tile_map: [0; 1024],
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
                // if temp_tile[y] != 0 && x < 10 {
                //     print!("Tile #{} byte {} is {:#x} \n", x / 16, y, temp_tile[y] );
                // }
            }
            // todo need to redo these so the output is those 8 commands about decoding
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
    pub fn set_stat_ppu_mode(&mut self, mbc: &mut Mbc, ppu_mode: PPUMode) {
        match ppu_mode {
            PPUMode::H_Blank => {
                mbc.hw_reg.stat &= 0b1111_1110;
            },
            PPUMode::V_Blank => {
                mbc.hw_reg.stat |= 0b0000_0001;
            },
            PPUMode::OAM_Scan => {
                mbc.hw_reg.stat |= 0b0000_0010;
            },
            PPUMode::Draw => {
                mbc.hw_reg.stat |= 0b0000_0011;
            },
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

    pub fn load_bg_tile_map(&mut self, mbc: &Mbc) {
        let address = if self.is_lcdc_bit3_bg_tile_map_set(&mbc) {
            0x9C00
        } else {
            0x9800
        };

        for x in 0x0..0x3FF {
            let temp = mbc.read(address + x);
            let idx = x as usize;
            self.bg_tile_map[idx] = temp;
            // if temp != 47 && temp != 0 {
            //     print!("Tile map entry is {:?} \n", temp);
            // }
        }
    }

    pub fn mode_2_oam_scan(&mut self) {
    
        // search for obj that are in this scan line pos and add to vec?
        
    }


    pub fn draw_tiles(&self, tw_buffer: &Arc<Mutex<Vec<u8>>>, cycles: &u64) {
        if !self.ppu_init_complete { return; }
        let mut pixel_count: usize = 0;
        let mut tile_in_grid_count: usize = 0;
        let rows_per_grid: usize = 8;
        let mut tile_in_row_count = 0;
        let tiles_per_row: usize = 16;
        let rows_per_tile = 8;
        let pixels_per_row = 8;
        let num_of_pixels_to_pad: usize = 32;
        let mut temp_buffer = vec![0u8; 92_160];

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
                            // for pi in 0..4 {
                            //     pixels_source[pi] = 255;
                            // }
                            temp_buffer[pixel_count..pixel_count+4].copy_from_slice(&[255; 4]);
                            pixel_count += 4;
                        }
                        tile_in_row_count = 0;
                        break;
                    } else {
                        // tile.data is an array of 8 arrays that each hold 8 PaletteColor
                        for pixel in 0..pixels_per_row {
                            //put each pixel into a vec so we can move it to the frame later
                            let rgba = tile.data[row][pixel].get_rgba_code();
                            temp_buffer[pixel_count..pixel_count+4].copy_from_slice(&rgba);
                            pixel_count += 4;
                        }
                        tile_in_row_count += 1;
                    }
                }
            }
        }
        {
            let mut tw_buffer_unlocked = tw_buffer.lock().unwrap();
            *tw_buffer_unlocked = temp_buffer;
        }

    }



    pub fn mode_3_draw(&self, gw_buffer: &Arc<Mutex<Vec<u8>>>, cycles: &u64) {
        // todo merge all the pixels from the pipe line here
        if !self.ppu_init_complete { return; }
        let mut temp_buffer = vec![0u8; 92_160];
        //let mut gw_buffer_unlocked = gw_buffer.lock().unwrap();
        // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile
        let mut pixel_count: usize = 0;
        let mut tile_in_grid_count: usize = 0;
        let rows_per_grid: usize = 18;
        let tiles_per_row: usize = 20;
        let rows_per_tile = 8;
        let pixels_per_row_in_tile = 8;
        for row_of_tiles_in_grid in 0..rows_per_grid {
            // take the first row of each tile, then second, etc
            for row_in_tile in 0..rows_per_tile {
                // loop 32 times so we get the index for each tile in the row of the grid
                for tpr in 0..tiles_per_row {
                    let mut tile_index = self.bg_tile_map[tile_in_grid_count + tpr] as usize;
                    // tile.data is an array of 8 arrays that each hold 8 PaletteColor
                    for pixel in 0..pixels_per_row_in_tile {
                        //put each pixel into a vec so we can move it to the frame later
                        let mut rgba = self.tiles[tile_index].data[row_in_tile][pixel].get_rgba_code();
                        // if rgba != [255, 255, 255, 255] {
                        //print!("rgba is {:?} \n", rgba);
                        // }
                        //rgba = [255, 0, 0, 255]; // testing if this will render
                        temp_buffer[pixel_count..pixel_count+4].copy_from_slice(&rgba);
                        pixel_count += 4;
                    }
                }
            }
            // inc every row in grid so we don't get the same tiles
            tile_in_grid_count += 20;
        }
        {
            let mut gw_buffer_unlocked = gw_buffer.lock().unwrap();
            *gw_buffer_unlocked = temp_buffer;
        }
    }

    pub fn draw_bgmap(&self, bgmw_buffer: &Arc<Mutex<Vec<u8>>>, cycles: &u64) {

        if !self.ppu_init_complete { return; }
        let mut temp_buffer = vec![0u8; 262_144];
        //let mut gw_buffer_unlocked = gw_buffer.lock().unwrap();
        // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile
        let mut pixel_count: usize = 0;
        let mut tile_in_grid_count: usize = 0;
        let rows_per_grid: usize = 32;
        let tiles_per_row: usize = 32;
        let rows_per_tile = 8;
        let pixels_per_row_in_tile = 8;
        for row_of_tiles_in_grid in 0..rows_per_grid {
            // take the first row of each tile, then second, etc
            for row_in_tile in 0..rows_per_tile {
                // loop 32 times so we get the index for each tile in the row of the grid
                for tpr in 0..tiles_per_row {
                    let mut tile_index = self.bg_tile_map[tile_in_grid_count + tpr] as usize;
                    // tile.data is an array of 8 arrays that each hold 8 PaletteColor
                    for pixel in 0..pixels_per_row_in_tile {
                        //put each pixel into a vec so we can move it to the frame later
                        let mut rgba = self.tiles[tile_index].data[row_in_tile][pixel].get_rgba_code();
                        // if rgba != [255, 255, 255, 255] {
                        //print!("rgba is {:?} \n", rgba);
                        // }
                        //rgba = [255, 0, 0, 255]; // testing if this will render
                        temp_buffer[pixel_count..pixel_count+4].copy_from_slice(&rgba);
                        pixel_count += 4;
                    }
                }
            }
            // inc every row in grid so we don't get the same tiles
            tile_in_grid_count += 32;
        }
        {
            let mut bgmw_buffer_unlocked = bgmw_buffer.lock().unwrap();
            *bgmw_buffer_unlocked = temp_buffer;
        }
    }

    pub fn start_interrupt_48() {

    }
    pub fn mode_0_h_blank(&self) {
        
    }

    pub fn mode_1_v_blank(&mut self, mbc: &mut Mbc) {
        // end the drawing of pixels in the ppu

    }

    pub fn tick(&mut self, mbc: &mut Mbc, tw: &Arc<Mutex<Vec<u8>>>, bgmw: &Arc<Mutex<Vec<u8>>>, gw: &Arc<Mutex<Vec<u8>>>, cycles: u64) -> PPUEvent {
        let tcycle = cycles * 4;
        
        // don't tick ppu unless the lcdc says ppu is on
        // i went back and forth here but I left it on because it seems like it may work
        // the pc counter was inc slow but that was due to other reasons
        if !mbc.hw_reg.is_lcdc_bit7_enabled() {
           //print!("lcdc bit 7 not enabled yet, skipping ppu tick \n");
            return PPUEvent::RenderEvent(RenderState::NoRender);
        }

        if mbc.hw_reg.lyc == mbc.hw_reg.ly {
            // set bit 2 when ly == lyc constantly
            mbc.hw_reg.set_stat_lyc_eq_ly_bit2();
            if mbc.hw_reg.is_stat_lyc_int_sel_bit6_set() {
                mbc.hw_reg.set_if_lcd_stat_bit1();
            }
        }
        else {
            // clear all except bit 2
            mbc.hw_reg.clear_stat_lyc_eq_ly_bit2();
        }


        if !self.ppu_init_complete {
            self.load_all_tiles(&mbc);
            self.load_bg_tile_map(&mbc);
            self.ppu_init_complete = true;
            print!("ppu init complete \n");
        }

        if mbc.need_tile_update {
            self.load_all_tiles(&mbc);
            //print!("need tile_update \n");
            mbc.need_tile_update = false;
        }

        if mbc.need_bg_map_update {
            self.load_bg_tile_map(&mbc);
            //print!("need bg_tile_map update \n");
            mbc.need_bg_map_update = false;
        }

        // go through all PPU modes
        // mode 2 + 3 + 0 stop after scan line 143
        self.tcycle_in_scanline += tcycle;
        self.tcycle_in_frame += tcycle;
        let mode_1_v_blank_first_scan_line = 144;
        let current_scanline = mbc.hw_reg.ly;
        let mode_0_h_blank_first_tcycle = 252;
        let mode_3_drawing_first_tcycle = 80;
        let mode_2_oam_scan_last_tcycle = 80;
        self.draw_tiles(tw, &tcycle);
        self.draw_bgmap(bgmw, &cycles);
        self.mode_3_draw(gw, &cycles);
        //let mode_2_oam_scan_last_cycle: u64 = 80;
        //print!("current scan line is {}\n", current_scanline);
        //print!("current tcycle_in_scanline is {}\n", self.tcycle_in_scanline);
        if current_scanline < mode_1_v_blank_first_scan_line {
            // mode 2 is dot 0-80
            if self.tcycle_in_scanline <  mode_2_oam_scan_last_tcycle {
                self.set_stat_ppu_mode(mbc, PPUMode::OAM_Scan);
            }
            if self.tcycle_in_scanline < mode_2_oam_scan_last_tcycle && !self.finished_mode_2_in_frame {
                self.mode_2_oam_scan();
                //print!("entering mode_2_oam_scan \n");
                // not updating tycle manually because I want the cpu and ppu in sync
                // self.tcycle_in_scanline = 79;
                self.finished_mode_2_in_frame = true;
            }
            
            if self.tcycle_in_scanline >  mode_3_drawing_first_tcycle && self.tcycle_in_scanline < mode_0_h_blank_first_tcycle {
                self.set_stat_ppu_mode(mbc, PPUMode::Draw);
            }
            if self.tcycle_in_scanline >= mode_3_drawing_first_tcycle  && !self.finished_mode_3_in_frame {
                // Mode 3 is between 172 and 289 dots, let's call it 172
                //print!("entering mode_3_drawing \n");
                // self.draw_tiles(tw, &tcycle);
                // self.draw_bgmap(bgmw, &cycles);
                // self.mode_3_draw(gw, &cycles);

                self.finished_mode_3_in_frame = true;
            }
            if self.tcycle_in_scanline >  mode_0_h_blank_first_tcycle {
                self.set_stat_ppu_mode(mbc, PPUMode::H_Blank);
            }
            if self.tcycle_in_scanline >= mode_0_h_blank_first_tcycle && !self.finished_mode_0_in_frame {
                //print!("entering mode_0_h_blank \n");
                // Mode 0 is the remainder of the dots left in the scan line (final dot is 456)
                self.mode_0_h_blank();
                self.finished_mode_0_in_frame = true;
            }
        } else {

            self.set_stat_ppu_mode(mbc, PPUMode::V_Blank);

            // last 10 scan lines are mode 1
            // 4560 dots or 10 scan lines (each scan line is 456 dots)

            let mode_1_v_blank_first_tcycle = 65664;
            if self.tcycle_in_frame >= mode_1_v_blank_first_tcycle && !self.finished_mode_1_in_frame {
                //mbc.hw_reg.set_ie_vblank_bit0();
                //print!("entering mode_1_v_blank \n");
                self.mode_1_v_blank(mbc);
                self.finished_mode_1_in_frame = true;
            }
        }

        //if all modes  are done cycle back
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
            return PPUEvent::RenderEvent(RenderState::Render);

        } else {
            return PPUEvent::RenderEvent(RenderState::NoRender);

        }
    }
}

