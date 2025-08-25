use crate::gb::emu::Emu;
use crate::gb::mbc::*;
use crate::gb::graphics::palette::*;
use crate::gb::graphics::tile::*;
use crate::gb::hwregisters::*;
use crate::gb::graphics::fetcher::*;
use crate::gb::graphics::fifo::*;
use crate::gb::gbwindow::*;

use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::Sender;
use std::thread::current;
use pixels::Pixels;
use crate::gb::graphics::pixel::GBPixel;
use crate::gb::graphics::sprite::Sprite;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Interrupt {
    Vblank_40,
    Stat_48,
    Timer_50,
    Serial_58,
    Joypad_60,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PPUEvent {
    RenderEvent(RenderState),
    InterruptEvent(Interrupt),
    EndOfScanLine,
    InitNotComplete,
    BufferOverflow,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderState {
    Render,
    NoRender
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PPUMode {
    Mode_0_H_Blank,
    Mode_1_V_Blank,
    Mode_2_OAM_Scan,
    Mode_3_Draw,
}



pub struct Ppu {
    fetcher: Fetcher,
    bg_win_fifo: Fifo,
    sprite_fifo: Fifo,
    tcycle_in_frame: u64,
    tcycle_in_scanline: u64,
    pixel_in_scanline: u64,
    started_mode_0_in_scanline: bool,
    started_mode_1_in_frame: bool,
    started_mode_2_in_scanline: bool,
    started_mode_3_in_scanline: bool,
    pub tiles: Vec<Tile>,
    pub sprites: Vec<Sprite>,
    //pub sprites_in_oam_idx: u16,
    //pub sprites_interesting_x_pos: [u8; 10],
    //bg_tile_map: [u8; 1024],
    pub ppu_init_complete: bool,
    pub bg_tile_map: [u8; 1024],
    // pub active: bool,
    pub tcycle_in_mode_3_draw: u64,
    pub pixel_in_frame: u64,
    drew_tiles_in_mode_3: bool,
    pub  mode_1_v_blank_first_scan_line: u8,
    pub  mode_0_h_blank_first_tcycle: u64,
    pub  mode_3_drawing_first_tcycle: u64,
    pub  mode_2_oam_scan_last_tcycle: u64,
    pub  mode_2_oam_scan_current_tcycle: u16,
    pub mode: PPUMode,
}
impl Ppu {
    pub fn new() -> Self {
        Ppu {
            fetcher: Fetcher::new(),
            bg_win_fifo: Fifo::new(),
            sprite_fifo: Fifo::new(),
            tcycle_in_frame: 0,
            tcycle_in_scanline: 0,
            pixel_in_scanline: 0,
            started_mode_0_in_scanline: false,
            started_mode_1_in_frame: false,
            started_mode_2_in_scanline: false,
            started_mode_3_in_scanline: false,
            tiles: Vec::new(),
            sprites: Vec::new(),
            //sprites_in_oam_idx: 0,
            //sprites_interesting_x_pos: [0; 10],
            ppu_init_complete: false,
            bg_tile_map: [0; 1024],
            // active: false,
            tcycle_in_mode_3_draw: 0,
            pixel_in_frame: 0,
            drew_tiles_in_mode_3: false,
            mode_1_v_blank_first_scan_line: 144,
            mode_0_h_blank_first_tcycle: 369,
            mode_3_drawing_first_tcycle: 80,
            mode_2_oam_scan_last_tcycle: 80,
            mode_2_oam_scan_current_tcycle: 0,
            mode: PPUMode::Mode_2_OAM_Scan,
        }
    }


    // deprecated
    pub fn load_all_tiles(&mut self, mbc: &Mbc) {
        self.tiles.clear();
        let address: u16 = 0x8000;

        // the whole tile range
        for x in (0..6144).step_by(16) {
            let mut new_tile = Tile::new();
            // every 16 bytes is a tile
            let mut temp_tile: [u8; 16] = [0; 16];
            for y in 0..16 {
                temp_tile[y] = mbc.read(address + x + (y as u16), OpSource::PPU);
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
            PPUMode::Mode_0_H_Blank => {
                mbc.hw_reg.stat &= 0b1111_1110;
            },
            PPUMode::Mode_1_V_Blank => {
                mbc.hw_reg.stat |= 0b0000_0001;
            },
            PPUMode::Mode_2_OAM_Scan => {
                mbc.hw_reg.stat |= 0b0000_0010;
            },
            PPUMode::Mode_3_Draw => {
                mbc.hw_reg.stat |= 0b0000_0011;
            },
        }
        self.mode = ppu_mode;
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
            let temp = mbc.read(address + x, OpSource::PPU);
            let idx = x as usize;
            self.bg_tile_map[idx] = temp;
            // if temp != 47 && temp != 0 {
            //     print!("Tile map entry is {:?} \n", temp);
            // }
        }
    }

    pub fn mode_2_oam_scan(&mut self, mbc: &mut Mbc, tcycles: u64) {
        let mode_2_max_tcycles: u16 = 80;
        let tcycle_per_oam: u16 = 2;

        // accumulate budget
        self.mode_2_oam_scan_current_tcycle += tcycles as u16;
        if self.mode_2_oam_scan_current_tcycle > mode_2_max_tcycles {
            self.mode_2_oam_scan_current_tcycle = mode_2_max_tcycles;
        }

        // how many sprite entries have we scanned so far?
        let current_entry = self.mode_2_oam_scan_current_tcycle.saturating_sub(tcycles as u16) / tcycle_per_oam;
        let final_entry   = self.mode_2_oam_scan_current_tcycle / tcycle_per_oam;

        // iterate only over the new entries
        for entry in current_entry..final_entry {
            if self.sprites.len() == 10 {
                break;
            }

            let addr = 0xFE00 + (entry * 4) as u16;
            let mut sprite = Sprite::default();
            sprite.byte0_y_pos        = mbc.read(addr + 0, OpSource::PPU);
            sprite.byte1_x_pos        = mbc.read(addr + 1, OpSource::PPU);
            sprite.byte2_tile_num     = mbc.read(addr + 2, OpSource::PPU);
            sprite.byte3_sprite_flags = mbc.read(addr + 3, OpSource::PPU);

            let sprite_y = sprite.byte0_y_pos;
            let sprite_x = sprite.byte1_x_pos;

            if sprite_x > 0 && sprite_y <= mbc.hw_reg.ly.wrapping_add(16) {
                let height = if mbc.hw_reg.is_lcdc_obj_size_bit2_enabled() { 16 } else { 8 };
                if sprite_y.wrapping_add(height) > mbc.hw_reg.ly.wrapping_add(16) {
                    self.sprites.push(sprite);
                    if self.sprites.len() == 10 {
                        break;
                    }
                }
            }
        }

        // sort once after scan is done or after hitting 10
        if self.mode_2_oam_scan_current_tcycle >= mode_2_max_tcycles || self.sprites.len() == 10 {
            self.sprites.sort_by(|a, b| a.byte1_x_pos.cmp(&b.byte1_x_pos));
        }
    }

    // pub fn mode_2_oam_scan(&mut self, mbc: &mut Mbc, tcycles: u64)   {
    //     // may not be needed since there is already an 80 tcycle delay upstream
    //
    //     let mode_2_max_tcycles: u16 = 80;
    //     let tcycle_per_oam: u16 = 2;
    //     if tcycles < tcycle_per_oam as u64 {
    //         // todo account for tcycle like 1 being saved in a budget once we validate that behavior should occur
    //         //self.mode_2_oam_scan_tcycle_budget += tcycles;
    //         return
    //     }
    //
    //     let current_loop_pos =  self.mode_2_oam_scan_current_tcycle / tcycle_per_oam;
    //     let final_loop_pos = (self.mode_2_oam_scan_current_tcycle + tcycles as u16) / tcycle_per_oam;
    //
    //     self.mode_2_oam_scan_current_tcycle += tcycles as u16;
    //
    //     let oam_address: u16 = 0xFE00;
    //
    //
    //     // the whole tile range
    //     for x in (current_loop_pos * 4..160).step_by(4) {
    //         if self.sprites.len() == 10 {
    //             //print!("sprites_in_oam_idx is 10, returning \n ");
    //             return;
    //         }
    //         if x >= final_loop_pos * 4 {
    //             return;
    //         }
    //         // every 4 bytes is a sprite
    //         // build the sprite
    //         let mut eligible_sprite: Sprite = Default::default();
    //
    //         //print!("checking sprite oam at {} \n ", pradd);
    //         eligible_sprite.byte0_y_pos             = mbc.read(oam_address + x + 0, OpSource::PPU);
    //         eligible_sprite.byte1_x_pos             = mbc.read(oam_address + x + 1, OpSource::PPU);
    //         eligible_sprite.byte2_tile_num          = mbc.read(oam_address + x + 2, OpSource::PPU);
    //         eligible_sprite.byte3_sprite_flags      = mbc.read(oam_address + x + 3, OpSource::PPU);
    //
    //         // bytes 0 to 3 are y pos, x pos, tile_idx, flags
    //         // check sprite parameters to determine if it needs to be drawn
    //         // sprite x pos greater than 0
    //         // sprite y pos <= ly + 16
    //         // sprite y pos + sprite height 8 or 16 >= lu + 16
    //         // if pass store in self.sprites (10 max)
    //         // if self.sprites is full, exit
    //         if eligible_sprite.byte1_x_pos > 0 && eligible_sprite.byte0_y_pos <= mbc.hw_reg.ly + 16 {
    //             // check for stacked tile or regular tile and add to array with idx
    //             if  (mbc.hw_reg.is_lcdc_obj_size_bit2_enabled() && eligible_sprite.byte0_y_pos + 16 > mbc.hw_reg.ly + 16)
    //                 || eligible_sprite.byte0_y_pos + 8 > mbc.hw_reg.ly + 16 {
    //                 //self.sprites_interesting_x_pos[self.sprites_in_oam_idx as usize] = eligible_sprite.byte1_x_pos;
    //                 //self.sprites[self.sprites_in_oam_idx as usize] = eligible_sprite;
    //                 self.sprites.push(eligible_sprite);
    //                 if self.sprites.len() == 10 || self.mode_2_oam_scan_current_tcycle >= 80 {
    //                     // sort the sprites by X pos because we want the left-most sprites first when we use them in the fetcher steps
    //                     // they're already filtered as being in the scanline, so we don't need to sort or filter by Y
    //                     self.sprites.sort_by(|x1, x2| x1.byte1_x_pos.cmp(&x2.byte1_x_pos));
    //                     return;
    //                 }
    //                 // self.sprites_in_oam_idx += 1;
    //                 // if self.sprites_in_oam_idx == 10 {
    //                 //
    //                 //     //print!("sprites_in_oam_idx is 10, returning \n ");
    //                 // }
    //             }
    //         }
    //     }
    //    // if  self.mode_2_oam_scan_current_tcycle >= 80 {
    //         // catch all sort
    //         self.sprites.sort_by(|x1, x2| x1.byte1_x_pos.cmp(&x2.byte1_x_pos));
    //     //}
    // }

    //
    // pub fn draw_tiles(&self, tw_buffer: &Arc<Mutex<Vec<u8>>>, cycles: &u64) {
    //     if !self.ppu_init_complete { return; }
    //     let mut rgba_count: usize = self.tcycle_in_mode_3_draw as usize * 4;
    //     let mut pixels_to_draw = cycles.clone();
    //     let mut pixels_drew: u64 = 0;
    //     let mut tile_in_grid_count: usize = 0;
    //     let rows_per_grid: usize = 15;
    //     let mut tile_in_row_count = 0;
    //     let tiles_per_row: usize = 16;
    //     let rows_per_tile = 8;
    //     let pixels_per_row = 8;
    //     let pixels_in_grid_row: u64 = 128;
    //     let num_of_pixels_to_pad: usize = 32;
    //     let mut tw_buffer_unlocked = tw_buffer.lock().unwrap();
    //
    //     let mut current_tcycles = self.tcycle_in_mode_3_draw;
    //     //print!("current_tcycles is {}\n", current_tcycles);
    //     //print!("cycles is {}\n", cycles);
    //
    //     let pixels_per_full_row_of_tiles: u64 = (pixels_per_row * rows_per_tile * tiles_per_row) as u64;
    //     //print!("pixels_per_full_row_of_tiles is {}\n", pixels_per_full_row_of_tiles);
    //     let starting_row_in_grid = current_tcycles / pixels_per_full_row_of_tiles;
    //     //print!("starting_row_in_grid is {}\n", starting_row_in_grid);
    //
    //     if current_tcycles >= pixels_per_full_row_of_tiles {
    //         current_tcycles -= starting_row_in_grid * pixels_per_full_row_of_tiles;
    //     }
    //     //print!("current_tcycles is {}\n", current_tcycles);
    //
    //     let mut starting_row_in_tile = current_tcycles / (rows_per_tile * tiles_per_row) as u64;
    //     //print!("starting_row_in_tile is {}\n", starting_row_in_tile);
    //
    //     let completed_tiles_pixels = starting_row_in_tile * rows_per_tile as u64 * tiles_per_row as u64;
    //     //print!("completed_tiles_pixels is {}\n", completed_tiles_pixels);
    //
    //     current_tcycles -= completed_tiles_pixels as u64;
    //
    //     let remaining_pixels = current_tcycles;
    //
    //     // print!("remaining_pixels is {}\n", remaining_pixels);
    //     // print!("pixels_to_draw is {}\n", pixels_to_draw);
    //
    //     let mut tile_num = (starting_row_in_grid as usize * tiles_per_row) + ((remaining_pixels as usize + 1) / pixels_per_row as usize);
    //     let mut pixels_to_skip: usize = (remaining_pixels as usize) % pixels_per_row as usize;
    //
    //     // if pixels_to_draw > 8 {
    //     //     //tile_num += (pixels_to_draw as usize / pixels_per_row);
    //     //     pixels_to_skip = (pixels_to_draw as usize % pixels_per_row);
    //     // }
    //
    //     if tile_num > 256 {
    //         //print!("skipping since the tile map is done\n");
    //         return;
    //     }
    //     //print!("tile_num is {}\n", tile_num);
    //     for row_of_tiles_in_grid in starting_row_in_grid as usize..rows_per_grid {
    //         for row in starting_row_in_tile as usize..rows_per_tile {
    //
    //             for tpr in 0..tiles_per_row {
    //                 //tile_in_grid_count = 0;
    //                 let tile = &self.tiles[tile_num + tpr];
    //
    //                 // tile.data is an array of 8 arrays that each hold 8 PaletteColor
    //                 for pixel in 0..pixels_per_row {
    //                     if pixels_to_skip > 0 { pixels_to_skip -= 1; continue; }
    //                     let rgba = tile.data[row][pixel].get_rgba_code();
    //                     tw_buffer_unlocked[rgba_count..rgba_count + 4].copy_from_slice(&rgba);
    //                     rgba_count += 4;
    //                     pixels_drew += 1;
    //                     if pixels_drew == pixels_to_draw {
    //                         return;
    //                     }
    //                 }
    //             }
    //         }
    //         // cant happen at begin bec. it breaks first run
    //         tile_num = ((row_of_tiles_in_grid as usize + 1) * tiles_per_row) ;
    //         starting_row_in_tile = 0;
    //         //print!("tile_num is {}\n", tile_num);
    //         if tile_num > 256 {
    //             //print!("skipping since the tile map is done\n");
    //             return;
    //         }
    //     }
    // }



    pub fn draw_tiles(&mut self, mbc: &mut Mbc, tw_buffer: &Arc<Mutex<Vec<u8>>>) {
        if !self.ppu_init_complete { return; }
        let mut pixel_count: usize = 0;
        let mut tile_in_grid_count: usize = 0;
        const ROWS_PER_GRID: usize = 8;
        let mut tile_in_row_count = 0;
        const TILES_PER_ROW: usize = 16;
        let rows_per_tile = 8;
        let pixels_per_row = 8;
        const TILE_COUNT: usize = ROWS_PER_GRID * TILES_PER_ROW;
        let num_of_pixels_to_pad: usize = 8;
        let mut temp_buffer = vec![0u8; 65_536];
        for row_of_tiles_in_grid in 0..ROWS_PER_GRID {
            for row in 0..rows_per_tile {
                tile_in_grid_count = 0;
                for tile_idx in 0..TILE_COUNT {
                    let tile = get_tile(mbc, tile_idx as u8, TileType::BG_WIN);
                    if row_of_tiles_in_grid > 0 {
                        if tile_in_grid_count < row_of_tiles_in_grid * TILES_PER_ROW {
                            tile_in_grid_count += 1;
                            continue;
                        }
                    }
                    // pad some bytes because the tiles don't take the whole screen
                    if tile_in_row_count == TILES_PER_ROW {
                        tile_in_row_count = 0;
                        break;
                    } else {
                        // tile.data is an array of 8 arrays that each hold 8 PaletteColor
                        for pixel in 0..pixels_per_row {
                            //put each pixel into a vec so we can move it to the frame later

                            let rgba = tile.data[row][pixel].get_rgba_code();
                            // let rgba = tile.data[row][pixel].get_rgba_code();
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

    // // // draws correctly but is NOT pixel per cycle
    // pub fn mode_3_draw(&self, gw_buffer: &Arc<Mutex<Vec<u8>>>, _cycles: &u64) {
    //     if !self.ppu_init_complete {
    //         return;
    //     }
    //
    //     const ROWS_PER_GRID: usize = 18;
    //     const TILES_PER_ROW: usize = 20;
    //     const ROWS_PER_TILE: usize = 8;
    //     const PIXELS_PER_ROW_IN_TILE: usize = 8;
    //     const RGBA_SIZE: usize = 4;
    //
    //     let mut temp_buffer = vec![0u8; ROWS_PER_GRID * ROWS_PER_TILE * TILES_PER_ROW * PIXELS_PER_ROW_IN_TILE * RGBA_SIZE];
    //     let mut rgba_index = 0;
    //     let mut tile_map_index = 0;
    //
    //     for row_of_tiles_in_grid in 0..ROWS_PER_GRID {
    //         for row_in_tile in 0..ROWS_PER_TILE {
    //             for tile_x in 0..TILES_PER_ROW {
    //                 let tile_index = self.bg_tile_map[tile_map_index + tile_x] as usize;
    //                 let tile_row = &self.tiles[tile_index].data[row_in_tile];
    //                 for &color in tile_row.iter() {
    //                     let rgba = color.get_rgba_code();
    //                     temp_buffer[rgba_index..rgba_index + 4].copy_from_slice(&rgba);
    //                     rgba_index += 4;
    //                 }
    //             }
    //         }
    //         tile_map_index += (TILES_PER_ROW + 12);
    //     }
    //
    //     let mut buffer = gw_buffer.lock().unwrap();
    //     *buffer = temp_buffer;
    // }

    pub fn push_pixel_and_advance_counter(&mut self, gw_buffer_unlocked: &mut MutexGuard<Vec<u8>>, px: GBPixel ) -> Result<(), PPUEvent> {
        // pushing px and fetching occur simultaneously
        //if self.fetcher.tcycle_budget == 0 { return Ok(()); }

        //let rgba = px.color.get_rgba_code();

        // todo validate logic of skipping pixels for sprites
        // return white if the pixel is to be skipped, this is for the horizontal per-pixel scrolling
        let rgba =  if !px.skip {
            px.color.get_rgba_code()
        } else {
            //print!("pixel is to be skipped\n");
            [0xFF, 0xFF, 0xFF, 0xFF]
        };
        gw_buffer_unlocked[(self.pixel_in_frame as usize) * 4..(self.pixel_in_frame as usize) * 4 + 4 ] .copy_from_slice(&rgba);
        self.pixel_in_frame += 1;
        self.pixel_in_scanline += 1;
        if self.pixel_in_scanline == 159 {
            self.pixel_in_scanline = 0;
            return Err(PPUEvent::EndOfScanLine)
        }
        Ok(())
    }
    // // // version that draws a pixel per cycle
    pub fn mode_3_mix_pixels_and_draw(&mut self, mbc: &mut Mbc, gw_buffer: &Arc<Mutex<Vec<u8>>>, tcycles: &u64) -> Result<(), PPUEvent> {

        if !self.ppu_init_complete { return Err(PPUEvent::InitNotComplete); }
        let buffer_len: u64 = 92160;
        if (self.pixel_in_frame * 4) + 4 >= buffer_len {
            //print!("PPU pixel buffer is too small in mode_3_mix_pixels_and_draw");
            return Err(PPUEvent::BufferOverflow)
        }
        let tcycle_budget = tcycles.clone();
        let mut gw_buffer_unlocked = gw_buffer.lock().unwrap();

        for x in 0..tcycle_budget {
            if x == 8 {
                // fifo is 16 pixels but we only want to pull in chunks of 8
                //print!("breaking out of mode_3_mix_pixels_and_draw for loop early because we popped u8 pixels\n");
                break;

            }
                match (self.bg_win_fifo.pop(), self.sprite_fifo.pop()) {
                (Ok(bg_px), Err(_)) => {
                    // push bg_px
                   self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, bg_px)?
                },
                (Err(_), Ok(sp_px)) => {
                    // push sp_px
                    self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, sp_px)?
                },
                (Ok(bg_px), Ok(sp_px)) => {
                    if bg_px.bg_priority && bg_px.color != PaletteColor::White {
                        // push_bg_px
                        self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, bg_px)?

                    } else {
                        // push sp_px
                        self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, sp_px)?

                    }
                },
                (Err(_), Err(_)) => {
                    // push none
                    break;
                }
            }
        }

    Ok(())

    }

    // // // // version that draws a pixel per cycle NOT integrated with fetcher
    // pub fn mode_3_draw(&mut self, mbc: &mut Mbc, gw_buffer: &Arc<Mutex<Vec<u8>>>, tcycles: &u64) {
    //     // todo merge all the pixels from the pipe line here
    //     if !self.ppu_init_complete { return; }
    //     //let mut temp_buffer = vec![0u8; 92_160];
    //     let mut gw_buffer_unlocked = gw_buffer.lock().unwrap();
    //     // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile
    //     //let mut rgba_count: usize = self.tcycle_in_mode_3_draw as usize;
    //     let mut rgba_count: usize = self.tcycle_in_mode_3_draw as usize * 4;
    //     let max_tcycle_in_mode_3_draw: u64 = 23_040;
    //     //let max_tcycle_in_mode_3_draw: u64 = 41_618;
    //     if rgba_count > max_tcycle_in_mode_3_draw as usize * 4 {
    //         print!("rgba count >max_tcycle_in_mode_3_draw 92_160 in mode_3_draw \n");
    //         return;
    //     }
    //     let mut pixels_to_draw = tcycles.clone();
    //     let mut pixels_drew: u64 = 0;
    //
    //     //let mut rgba_count: usize = 0;
    //     let mut tile_in_grid_count: usize = 0;
    //     const ROWS_PER_GRID: usize = 18;
    //     const TILES_PER_ROW: usize = 20;
    //     const TILES_PER_ROW_IN_BG_GRID: usize = 32;
    //     const ROWS_PER_TILE: usize = 8;
    //     const PIXELS_PER_ROW: usize = 8;
    //
    //     let mut current_tcycles = self.tcycle_in_mode_3_draw;
    //     // print!("\ncurrent_tcycles is {}\n", current_tcycles);
    //     // print!("tcycles is {}\n", tcycles);
    //
    //     let pixels_per_full_row_of_tiles: u64 = (PIXELS_PER_ROW * ROWS_PER_TILE * TILES_PER_ROW) as u64;
    //     //print!("pixels_per_full_row_of_tiles is {}\n", pixels_per_full_row_of_tiles);
    //     let starting_row_in_grid = current_tcycles / pixels_per_full_row_of_tiles;
    //     //print!("starting_row_in_grid is {}\n", starting_row_in_grid);
    //
    //     if current_tcycles >= pixels_per_full_row_of_tiles {
    //         current_tcycles -= starting_row_in_grid * pixels_per_full_row_of_tiles;
    //     }
    //     //print!("current_tcycles is {}\n", current_tcycles);
    //
    //     let mut starting_row_in_tile = current_tcycles / (ROWS_PER_TILE * TILES_PER_ROW) as u64;
    //     //print!("starting_row_in_tile is {}\n", starting_row_in_tile);
    //
    //     let completed_tiles_pixels = starting_row_in_tile * (ROWS_PER_TILE * TILES_PER_ROW) as u64;
    //     //print!("completed_tiles_pixels is {}\n", completed_tiles_pixels);
    //
    //     current_tcycles -= completed_tiles_pixels as u64;
    //
    //     let remaining_pixels = current_tcycles;
    //
    //     // print!("remaining_pixels is {}\n", remaining_pixels);
    //     // print!("pixels_to_draw is {}\n", pixels_to_draw);
    //
    //     let mut tile_num = (starting_row_in_grid as usize * TILES_PER_ROW_IN_BG_GRID) + ((remaining_pixels as usize + 1) / PIXELS_PER_ROW as usize);
    //     let mut pixels_to_skip: usize = (remaining_pixels as usize) % PIXELS_PER_ROW as usize;
    //     // tile_num is assuming 32 tiles per row in grid, but our game only shows 20 tiles per row in grid, so we need to offset it
    //
    //     for row_of_tiles_in_grid in starting_row_in_grid as usize..ROWS_PER_GRID {
    //         // take the first row of each tile, then second, etc
    //         for row_in_tile in starting_row_in_tile as usize..ROWS_PER_TILE {
    //             // loop 32 times so we get the index for each tile in the row of the grid
    //             for tpr in 0..TILES_PER_ROW {
    //                 //let mut tile_index = self.bg_tile_map[tile_num + tpr] as usize;
    //                 let mut tile_index = self.get_index_from_bg_tile_map(mbc, tile_num + tpr);
    //                 // tile.data is an array of 8 arrays that each hold 8 PaletteColor
    //                 for pixel in 0..PIXELS_PER_ROW {
    //                     if pixels_to_skip > 0 { pixels_to_skip -= 1; continue; }
    //                     //put each pixel into a vec so we can move it to the frame later
    //                     //let mut rgba = self.tiles[tile_index].data[row_in_tile][pixel].get_rgba_code();
    //                     let tile = get_tile(mbc, tile_index as u8, TileType::BG_WIN);
    //                     let rgba = tile.data[row_in_tile][pixel].get_rgba_code();
    //                     if rgba_count >= max_tcycle_in_mode_3_draw as usize * 4 {
    //                         return;
    //                     }
    //                     gw_buffer_unlocked[rgba_count..rgba_count+4].copy_from_slice(&rgba);
    //                     rgba_count += 4;
    //                     pixels_drew += 1;
    //                     if pixels_drew == pixels_to_draw {
    //                         return;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }




    // pub fn draw_bg_map(&self, bgmw_buffer: &Arc<Mutex<Vec<u8>>>, tcycles: &u64) {
    //     // running out of cycles?
    //     if !self.ppu_init_complete { return; }
    //     let mut bgmw_buffer_unlocked = bgmw_buffer.lock().unwrap();
    //     // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile
    //     let mut rgba_count: usize = self.tcycle_in_mode_3_draw as usize * 4;
    //     let mut pixels_to_draw = tcycles.clone();
    //     let mut pixels_drew: u64 = 0;
    //     let rows_per_grid: usize = 32;
    //     let tiles_per_row: usize = 32;
    //     let rows_per_tile = 8;
    //     let pixels_per_row = 8;
    //
    //
    //     let mut current_tcycles = self.tcycle_in_mode_3_draw;
    //     // if current_tcycles >= 18_432 {
    //     //     print!("\ncurrent_tcycles is {}\n", current_tcycles);
    //     // }
    //    // print!("\ncurrent_tcycles is {}\n", current_tcycles);
    //    // print!("tcycles is {}\n", tcycles);
    //
    //     let pixels_per_full_row_of_tiles: u64 = (pixels_per_row * rows_per_tile * tiles_per_row) as u64;
    //     //print!("pixels_per_full_row_of_tiles is {}\n", pixels_per_full_row_of_tiles);
    //     let starting_row_in_grid = current_tcycles / pixels_per_full_row_of_tiles;
    //     //print!("starting_row_in_grid is {}\n", starting_row_in_grid);
    //
    //     if current_tcycles >= pixels_per_full_row_of_tiles {
    //         current_tcycles -= starting_row_in_grid * pixels_per_full_row_of_tiles;
    //     }
    //     //print!("current_tcycles is {}\n", current_tcycles);
    //
    //     let mut starting_row_in_tile = current_tcycles / (rows_per_tile * tiles_per_row) as u64;
    //     //print!("starting_row_in_tile is {}\n", starting_row_in_tile);
    //
    //     let completed_tiles_pixels = starting_row_in_tile * rows_per_tile as u64 * tiles_per_row as u64;
    //     //print!("completed_tiles_pixels is {}\n", completed_tiles_pixels);
    //
    //     current_tcycles -= completed_tiles_pixels as u64;
    //
    //     let remaining_pixels = current_tcycles;
    //
    //     // print!("remaining_pixels is {}\n", remaining_pixels);
    //     // print!("pixels_to_draw is {}\n", pixels_to_draw);
    //
    //     let mut tile_num = (starting_row_in_grid as usize * tiles_per_row) + ((remaining_pixels as usize + 1) / pixels_per_row as usize);
    //     let mut pixels_to_skip: usize = (remaining_pixels as usize) % pixels_per_row as usize;
    //
    //     // if pixels_to_draw > 8 {
    //     //     //tile_num += (pixels_to_draw as usize / pixels_per_row);
    //     //     pixels_to_skip = (pixels_to_draw as usize % pixels_per_row);
    //     // }
    //
    //     // if tile_num > 256 {
    //     //     //print!("skipping since the tile map is done\n");
    //     //     return;
    //     // }
    //
    //     for row_of_tiles_in_grid in starting_row_in_grid as usize..rows_per_grid {
    //         // take the first row of each tile, then second, etc
    //         for row_in_tile in starting_row_in_tile as usize..rows_per_tile {
    //             // loop 32 times so we get the index for each tile in the row of the grid
    //             for tpr in 0..tiles_per_row {
    //                 let mut tile_index = self.bg_tile_map[tile_num + tpr] as usize; // todo get the count right
    //                 // tile.data is an array of 8 arrays that each hold 8 PaletteColor
    //                 for pixel in 0..pixels_per_row {
    //                     if pixels_to_skip > 0 { pixels_to_skip -= 1; continue; }
    //                     //put each pixel into a vec so we can move it to the frame later
    //                     let mut rgba = self.tiles[tile_index].data[row_in_tile][pixel].get_rgba_code();
    //                     // if rgba != [255, 255, 255, 255] {
    //                     //     print!("rgba is {:?} \n", rgba);
    //                     // }
    //                     //rgba = [255, 0, 0, 255]; // testing if this will render
    //                     bgmw_buffer_unlocked[rgba_count..rgba_count+4].copy_from_slice(&rgba);
    //                     rgba_count += 4;
    //                     pixels_drew += 1;
    //                     if pixels_drew == pixels_to_draw {
    //                         return;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn get_index_from_bg_tile_map(&mut self, mbc: &Mbc, bg_idx: usize) -> usize {
        let address = if self.is_lcdc_bit3_bg_tile_map_set(&mbc) {
            0x9C00
        } else {
            0x9800
        };

        mbc.read(address + bg_idx as u16, OpSource::PPU) as usize
    }

    pub fn draw_bg_map(&mut self, mbc: &mut Mbc, bgmw_buffer: &Arc<Mutex<Vec<u8>>>) {
        if !self.ppu_init_complete {
            return;
        }

        const ROWS_PER_GRID: usize = 32;
        const TILES_PER_ROW: usize = 32;
        const ROWS_PER_TILE: usize = 8;
        const PIXELS_PER_ROW_IN_TILE: usize = 8;
        const RGBA_SIZE: usize = 4;

        let mut temp_buffer = vec![0u8; ROWS_PER_GRID * ROWS_PER_TILE * TILES_PER_ROW * PIXELS_PER_ROW_IN_TILE * RGBA_SIZE];
        let mut rgba_index = 0;
        let mut tile_map_index = 0;

        for row_of_tiles_in_grid in 0..ROWS_PER_GRID {
            for row_in_tile in 0..ROWS_PER_TILE {
                for tile_x in 0..TILES_PER_ROW {
                    //let tile_index = self.bg_tile_map[tile_map_index + tile_x] as usize;
                    let tile_index = self.get_index_from_bg_tile_map(mbc,tile_map_index + tile_x);
                    let tile = get_tile(mbc, tile_index as u8, TileType::BG_WIN);
                    let tile_row = tile.data[row_in_tile];
                    for &color in tile_row.iter() {
                        let rgba = color.get_rgba_code();
                        temp_buffer[rgba_index..rgba_index + 4].copy_from_slice(&rgba);
                        rgba_index += 4;
                    }
                }
            }
            tile_map_index += TILES_PER_ROW;
        }

        let mut buffer = bgmw_buffer.lock().unwrap();
        *buffer = temp_buffer;
    }
    // pub fn draw_bg_map(&self, bgmw_buffer: &Arc<Mutex<Vec<u8>>>, cycles: &u64) {
    //
    //     if !self.ppu_init_complete { return; }
    //     let mut temp_buffer = vec![0u8; 262_144];
    //     //let mut gw_buffer_unlocked = gw_buffer.lock().unwrap();
    //     // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile
    //     let mut rgba_count: usize = 0;
    //     let mut tile_in_grid_count: usize = 0;
    //     let rows_per_grid: usize = 32;
    //     let tiles_per_row: usize = 32;
    //     let rows_per_tile = 8;
    //     let pixels_per_row_in_tile = 8;
    //     for row_of_tiles_in_grid in 0..rows_per_grid {
    //         // take the first row of each tile, then second, etc
    //         for row_in_tile in 0..rows_per_tile {
    //             // loop 32 times so we get the index for each tile in the row of the grid
    //             for tpr in 0..tiles_per_row {
    //                 let mut tile_index = self.bg_tile_map[tile_in_grid_count + tpr] as usize;
    //                 // tile.data is an array of 8 arrays that each hold 8 PaletteColor
    //                 for pixel in 0..pixels_per_row_in_tile {
    //                     //put each pixel into a vec so we can move it to the frame later
    //                     let mut rgba = self.tiles[tile_index].data[row_in_tile][pixel].get_rgba_code();
    //                     // if rgba != [255, 255, 255, 255] {
    //                     //print!("rgba is {:?} \n", rgba);
    //                     // }
    //                     //rgba = [255, 0, 0, 255]; // testing if this will render
    //                     temp_buffer[rgba_count..rgba_count+4].copy_from_slice(&rgba);
    //                     rgba_count += 4;
    //                 }
    //             }
    //         }
    //         // inc every row in grid so we don't get the same tiles
    //         tile_in_grid_count += 32;
    //     }
    //     {
    //         let mut bgmw_buffer_unlocked = bgmw_buffer.lock().unwrap();
    //         *bgmw_buffer_unlocked = temp_buffer;
    //     }
    // }

    pub fn start_interrupt_48() {

    }
    pub fn mode_0_h_blank(&self, cycles: &u64) {
        
    }

    pub fn mode_1_v_blank(&mut self, mbc: &mut Mbc, cycles: &u64) {
        // end the drawing of pixels in the ppu

    }

    pub fn tick(&mut self, mbc: &mut Mbc, tw: &Arc<Mutex<Vec<u8>>>, bgmw: &Arc<Mutex<Vec<u8>>>, gw: &Arc<Mutex<Vec<u8>>>, cycles: u64) -> PPUEvent {
        let tcycle = cycles * 4;
        
        // don't tick ppu unless the lcdc says ppu is on
        // i went back and forth here but I left it on because it seems like it may work
        // the pc counter was inc slow but that was due to other reasons
        if !mbc.hw_reg.is_lcdc_lcd_and_ppu_enable_bit7_enabled() {
           //print!("lcdc bit 7 not enabled yet, skipping ppu tick \n");
            mbc.hw_reg.ly = 0;
            self.tcycle_in_scanline = 0;
            self.tcycle_in_frame = 0;
            //mbc.hw_reg.stat = 0;
            // Clear mode bits (0–1) and coincidence flag (bit 2) but preserve interrupt enable bits (3,4,5,6):
            mbc.hw_reg.stat = mbc.hw_reg.stat & 0b1111_1000;
            return PPUEvent::RenderEvent(RenderState::NoRender);
        }

        // if mbc.hw_reg.lyc == mbc.hw_reg.ly {
        //     // set bit 2 when ly == lyc constantly
        //     mbc.hw_reg.set_stat_lyc_eq_ly_bit2();
        //     if mbc.hw_reg.is_stat_lyc_int_sel_bit6_set() {
        //         mbc.hw_reg.set_if_lcd_stat_bit1();
        //     }
        // }
        // else {
        //     // clear all except bit 2
        //     mbc.hw_reg.clear_stat_lyc_eq_ly_bit2();
        // }
        // Handle LYC=LY coincidence
        let prev_coincidence = mbc.hw_reg.is_stat_lyc_eq_ly_bit2_set();

        if mbc.hw_reg.lyc == mbc.hw_reg.ly {
            mbc.hw_reg.set_stat_lyc_eq_ly_bit2();
            if !prev_coincidence && mbc.hw_reg.is_stat_lyc_int_sel_bit6_set() {
                mbc.hw_reg.set_if_lcd_stat_bit1(); // fire interrupt only on transition
            }
        } else {
            mbc.hw_reg.clear_stat_lyc_eq_ly_bit2();
        }

        if !self.ppu_init_complete {
            // self.load_all_tiles(&mbc);
            // self.load_bg_tile_map(&mbc);
            self.ppu_init_complete = true;
            print!("ppu init complete \n");
        }

        // if mbc.need_tile_update {
        //     //self.load_all_tiles(&mbc);
        //     //print!("need tile_update \n");
        //     mbc.need_tile_update = false;
        // }
        
        // if mbc.need_bg_map_update {
        //     //self.load_bg_tile_map(&mbc);
        //     //print!("need bg_tile_map update \n");
        //     mbc.need_bg_map_update = false;
        // }


        // go through all PPU modes
        // mode 2 + 3 + 0 stop after scan line 143
        self.tcycle_in_scanline += tcycle;
        self.tcycle_in_frame += tcycle;
        // these will be changes in mode 3 if it finishes early

        let current_scanline = mbc.hw_reg.ly;
        //moved the mode 0-3 limits to self so that I can change mode 0 and keep its state




        //let mode_2_oam_scan_last_cycle: u64 = 80;
        //print!("current scan line is {}\n", current_scanline);
        //print!("current tcycle_in_scanline is {}\n", self.tcycle_in_scanline);
        if current_scanline < self.mode_1_v_blank_first_scan_line {

            let mode_2_first_dot: u64 = 0;
            let current_dot = self.tcycle_in_scanline - tcycle;
            // set the PPU mode when entering a new mode
            //if self.tcycle_in_scanline < self.mode_2_oam_scan_last_tcycle && !self.started_mode_2_in_frame {
            if current_dot == mode_2_first_dot && !self.started_mode_2_in_scanline && !self.started_mode_3_in_scanline && !self.started_mode_0_in_scanline && !self.started_mode_1_in_frame {
                 // print!("entering mode_2_oam_scan \n");
                //reset oam idx so we check it every scan line
                //self.sprites_in_oam_idx = 0;
                // todo switch to using these in mode 2 as it's more accurate according to pandocs
                // requires modifying the logic in step one of the tile num in fetcher
                // ly == wy is only checked in start of mode 2 and is used later in the bg/win fifo process
                // if mbc.hw_reg.ly == mbc.hw_reg.wy {
                //     self.fetcher.window_layer_active_in_lcdc = true;
                //     self.fetcher.switched_to_window_layer = false;
                // }
                // todo implement start of rendering scan line
                // if start of rendering scan line
                // delay rendering 6 tcycle so the PPU can fetch the first tile's data
                // delay rendering 8 more tcycles because PPU does a fake render of this tile
                self.fetcher.start_of_rendering = true; // outside of if
                self.set_stat_ppu_mode(mbc, PPUMode::Mode_2_OAM_Scan);
                // need to skip this many pixels when rendering, mark them skipped in fetcher and skip in mode_3_draw
                self.fetcher.pixels_to_mark_skipped = mbc.hw_reg.scx % 8;
                // clear out oam data
                //self.sprites_in_oam_idx = 0;
                // always reset pixels in frame because that's what the buffer writer uses as an index
                self.pixel_in_scanline = 0;
                self.fetcher.finished_sprites_in_scanline = false;
                //clear it every scanline
                self.sprites.clear();
                self.sprites.reserve(10);
                self.mode_2_oam_scan_current_tcycle = 0;

                self.started_mode_2_in_scanline = true;
            }

            // mode 2 is dot 0-80
            if self.tcycle_in_scanline <  self.mode_2_oam_scan_last_tcycle && self.started_mode_2_in_scanline && !self.started_mode_3_in_scanline && !self.started_mode_0_in_scanline && !self.started_mode_1_in_frame {
                // pause for 80 tcycles
                self.mode_2_oam_scan(mbc, tcycle);

            }


            if self.tcycle_in_scanline >= self.mode_3_drawing_first_tcycle  && self.started_mode_2_in_scanline && !self.started_mode_3_in_scanline && !self.started_mode_0_in_scanline && !self.started_mode_1_in_frame {
                // Mode 3 is between 172 and 289 dots, let's call it 172
                 // print!("entering mode_3_draw \n");
                mbc.restrict_vram_access = true;
                //reset fifos
                self.sprite_fifo.data.clear();
                self.bg_win_fifo.data.clear();
                self.fetcher.tcycle_budget = 0;
                //self.fetcher.tile_x_coord = 0;
                // always reset the layer before we start
                self.fetcher.active_layer = Layer::BG;
                self.set_stat_ppu_mode(mbc, PPUMode::Mode_3_Draw);
                self.started_mode_3_in_scanline = true;
            }


            if self.tcycle_in_scanline >=  self.mode_3_drawing_first_tcycle && self.tcycle_in_scanline < self.mode_0_h_blank_first_tcycle && self.started_mode_2_in_scanline && self.started_mode_3_in_scanline && !self.started_mode_0_in_scanline && !self.started_mode_1_in_frame {

                //only draw tiles and bg_map once per mode 3 to reduce utilization
                if !self.drew_tiles_in_mode_3 {
                    self.draw_tiles(mbc, tw);
                    self.draw_bg_map(mbc, bgmw);
                    self.drew_tiles_in_mode_3 = true;
                }
                // todo fix why this is reaching > 255
                let tcycles_res = self.fetcher.tcycle_budget.overflowing_add(tcycle as u8);
                self.fetcher.tcycle_budget = if tcycles_res.1 {
                    print!("self.fetcher.tcycle_budget overflowed\n");
                    255
                } else {
                    tcycles_res.0
                };

                match self.fetcher.active_layer {
                    Layer::BG | Layer::WIN => {
                        //print!("matched BG layer in ppu.tick \n");
                        self.fetcher.handle_bg_win_layer(mbc, &mut self.bg_win_fifo, &mut self.sprite_fifo, &mut self.sprites, tcycle);
                    },
                    Layer::SPRITE => {
                        //print!("matched sprite layer in ppu.tick \n");
                        self.fetcher.handle_sprite_layer(mbc, &mut self.sprite_fifo, &mut self.sprites, tcycle);
                    },
                }

                match self.mode_3_mix_pixels_and_draw(mbc, gw, &tcycle) {
                    Ok(_) => {},
                    Err(PPUEvent::EndOfScanLine) => {
                        //print!("finished scan line early, switching to mode 0 H blank \n");
                        self.mode_0_h_blank_first_tcycle = self.tcycle_in_scanline;
                    },
                    Err(PPUEvent::BufferOverflow) => {
                       // reset everything since we finished the pixels

                    },
                    _=> { }
                }
                //self.tcycle_in_mode_3_draw += tcycle;
            }


            if self.tcycle_in_scanline >= self.mode_0_h_blank_first_tcycle && self.started_mode_2_in_scanline && self.started_mode_3_in_scanline && !self.started_mode_0_in_scanline && !self.started_mode_1_in_frame {
                 // print!("entering mode_0_h_blank \n");
                mbc.restrict_vram_access = false;

                // Mode 0 is the remainder of the dots left in the scan line (final dot is 456)
                self.set_stat_ppu_mode(mbc, PPUMode::Mode_0_H_Blank);
                self.started_mode_0_in_scanline = true;
            }
            if self.tcycle_in_scanline >=  self.mode_0_h_blank_first_tcycle && self.started_mode_2_in_scanline && self.started_mode_3_in_scanline && self.started_mode_0_in_scanline && !self.started_mode_1_in_frame {

                self.mode_0_h_blank(&tcycle);
            }
        }
        else { // scanline must be 144 or greater
            // last 10 scan lines are mode 1
            // 4560 dots or 10 scan lines (each scan line is 456 dots)
            //print!("tcycle in frame is {} and mode 1, 2, 3, and 0 bool are {}, {}, {}, {}\n", self.tcycle_in_frame, self.started_mode_1_in_frame, self.started_mode_2_in_scanline, self.started_mode_3_in_scanline, self.started_mode_0_in_scanline );
            let mode_1_v_blank_first_tcycle = 65_664;
            //if self.tcycle_in_frame >= mode_1_v_blank_first_tcycle && !self.started_mode_1_in_frame && self.started_mode_2_in_scanline && self.started_mode_3_in_scanline && self.started_mode_0_in_scanline {
            if self.tcycle_in_frame >= mode_1_v_blank_first_tcycle && !self.started_mode_1_in_frame  {
                mbc.hw_reg.set_if_vblank_bit0();
                 //print!("entering mode_1_v_blank \n");
                self.fetcher.win_y_pos = 0;
                self.fetcher.tile_y_pos = 0;
                // reset tile x pos every frame. It's & with 0x1F in the fetcher step 1
                self.fetcher.tile_x_pos = 0;
                //only draw these tiles once per frame in mode 3

                self.drew_tiles_in_mode_3 = false;

                self.set_stat_ppu_mode(mbc, PPUMode::Mode_1_V_Blank);
                self.started_mode_1_in_frame = true;
            }

            if self.started_mode_1_in_frame {
                self.mode_1_v_blank(mbc, &tcycle);
            }
        }

        // // //if all modes  are done cycle back
        // if self.started_mode_2_in_frame && self.started_mode_3_in_frame &&
        //         self.started_mode_0_in_frame && self.started_mode_1_in_frame {
        //     self.started_mode_2_in_frame = false;
        //     self.started_mode_3_in_frame = false;
        //     self.started_mode_0_in_frame = false;
        //     self.started_mode_1_in_frame = false;
        // }

        // reset tcycle in scan line because max is 456
        // also inc LY
        // todo switch to tcycle_in_scanline % 456 and save that as budget for next scanline?
        if self.tcycle_in_scanline >= 456 {
            // this print is very freq
            //print!("tcycle_in_scanline >= 456, incrementing LY \n");
            self.tcycle_in_scanline = 0;
            self.pixel_in_scanline = 0;
            self.started_mode_2_in_scanline = false;
            self.started_mode_3_in_scanline = false;
            self.started_mode_0_in_scanline = false;
            mbc.hw_reg.ly += 1;
        }
        // max ly is 153 because there are 153 scanlines
        let max_ly_value = 153;
        if mbc.hw_reg.ly > max_ly_value {
            mbc.hw_reg.ly = 0;
            self.tcycle_in_frame = 0;
            self.started_mode_2_in_scanline = false;
            self.started_mode_3_in_scanline = false;
            self.started_mode_0_in_scanline = false;
            self.started_mode_1_in_frame = false;
            // this print freq is the same as the 1 sec pausing, that means the ppu and cpu are in sync
            //print!("ly hw reg is max, resetting to 0 \n");
        }

        let max_draw_ly_value = 144;
        if mbc.hw_reg.ly > max_draw_ly_value {
            self.pixel_in_frame = 0;
        }

        //let max_tcycle_in_mode_3_draw: u64 = 23_040;
        //let max_tcycle_in_mode_3_draw: u64 = 24_768;
        //let max_tcycle_in_mode_3_draw: u64 = 41_616;

        let max_tcycle_in_frame = 70_224;
        // if self.tcycle_in_mode_3_draw >= max_tcycle_in_mode_3_draw {
        //     self.tcycle_in_mode_3_draw = 0;
        // }

        if self.tcycle_in_frame >= max_tcycle_in_frame {
            //print!("tcycle_in_frame is >= 70224, generating frame \n");
            self.tcycle_in_frame = 0;
            //self.tcycle_in_scanline = 0;
            //self.tcycle_in_mode_3_draw = 0;
            PPUEvent::RenderEvent(RenderState::Render)
        } else {
            return PPUEvent::RenderEvent(RenderState::NoRender)
        }

    }
}