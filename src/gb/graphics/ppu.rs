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
use pixels::Pixels;
use crate::gb::graphics::pixel::GBPixel;
use crate::gb::graphics::sprite::Sprite;

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
    FinishedScanLine
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
    fetcher: Fetcher,
    bg_win_fifo: Fifo,
    sprite_fifo: Fifo,
    tcycle_in_frame: u64,
    tcycle_in_scanline: u64,
    dot_in_scanline: u64,
    started_mode_0_in_frame: bool,
    started_mode_1_in_frame: bool,
    started_mode_2_in_frame: bool,
    started_mode_3_in_frame: bool,
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
}
impl Ppu {
    pub fn new() -> Self {
        Ppu {
            fetcher: Fetcher::new(),
            bg_win_fifo: Fifo::new(),
            sprite_fifo: Fifo::new(),
            tcycle_in_frame: 0,
            tcycle_in_scanline: 0,
            dot_in_scanline: 0,
            started_mode_0_in_frame: false,
            started_mode_1_in_frame: false,
            started_mode_2_in_frame: false,
            started_mode_3_in_frame: false,
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
            let temp = mbc.read(address + x, OpSource::PPU);
            let idx = x as usize;
            self.bg_tile_map[idx] = temp;
            // if temp != 47 && temp != 0 {
            //     print!("Tile map entry is {:?} \n", temp);
            // }
        }
    }

    pub fn mode_2_oam_scan(&mut self, mbc: &mut Mbc, cycles: &u64)   {
        // todo each am entry should be 2 tcycles, introduce checking and delay for that
        // may not be needed since there is already an 80 tcycle delay upstream

        //clear it every scanline
        self.sprites.clear();
        self.sprites.reserve(10);

        let oam_address: u16 = 0xFE00;

        // the whole tile range
        for x in (0..160).step_by(4) {
            if self.sprites.len() == 10 {
                //print!("sprites_in_oam_idx is 10, returning \n ");
                return;
            }
            // every 4 bytes is a sprite
            // build the sprite
            let mut eligible_sprite: Sprite = Default::default();
            let pradd = oam_address + x;
            //print!("checking sprite oam at {} \n ", pradd);
            eligible_sprite.byte0_y_pos             = mbc.read(oam_address + x + 0, OpSource::PPU);
            eligible_sprite.byte1_x_pos             = mbc.read(oam_address + x + 1, OpSource::PPU);
            eligible_sprite.byte2_tile_num          = mbc.read(oam_address + x + 2, OpSource::PPU);
            eligible_sprite.byte3_sprite_flags      = mbc.read(oam_address + x + 3, OpSource::PPU);

            // bytes 0 to 3 are y pos, x pos, tile_idx, flags
            // check sprite parameters to determine if it needs to be drawn
            // sprite x pos greater than 0
            // sprite y pos <= ly + 16
            // sprite y pos + sprite height 8 or 16 >= lu + 16
            // if pass store in self.sprites (10 max)
            // if self.sprites is full, exit
            if eligible_sprite.byte1_x_pos > 0 && eligible_sprite.byte0_y_pos <= mbc.hw_reg.ly + 16 {
                // check for stacked tile or regular tile and add to array with idx
                if  (mbc.hw_reg.is_lcdc_obj_size_bit2_enabled() && eligible_sprite.byte0_y_pos + 16 >= mbc.hw_reg.ly + 16)
                    || eligible_sprite.byte0_y_pos + 8 >= mbc.hw_reg.ly + 8 {
                    //self.sprites_interesting_x_pos[self.sprites_in_oam_idx as usize] = eligible_sprite.byte1_x_pos;
                    //self.sprites[self.sprites_in_oam_idx as usize] = eligible_sprite;
                    self.sprites.push(eligible_sprite);
                    if self.sprites.len() == 10 {
                        // sort the sprites by X pos because we want the left-most sprites first when we use them in the fetcher steps
                        // they're already filtered as being in the scanline, so we don't need to sort or filter by Y
                        self.sprites.sort_by(|x1, x2| x1.byte1_x_pos.cmp(&x2.byte1_x_pos));
                        return;
                    }
                    // self.sprites_in_oam_idx += 1;
                    // if self.sprites_in_oam_idx == 10 {
                    //
                    //     //print!("sprites_in_oam_idx is 10, returning \n ");
                    // }
                }
            }
        }
        // catch all sort
        self.sprites.sort_by(|x1, x2| x1.byte1_x_pos.cmp(&x2.byte1_x_pos));
    }

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
                            let tile = get_tile(mbc, tile_idx as u8, TileType::BG_WIN);
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
        let rgba = px.color.get_rgba_code();
        gw_buffer_unlocked[(self.pixel_in_frame as usize) * 4..(self.pixel_in_frame as usize) * 4 + 4 ] .copy_from_slice(&rgba);
        self.pixel_in_frame += 1;
        self.dot_in_scanline += 1;
        if self.dot_in_scanline == 160 {
            self.dot_in_scanline = 0;
            return Err(PPUEvent::FinishedScanLine)
        }
        Ok(())
    }
    // // // version that draws a pixel per cycle
    pub fn mode_3_mix_pixels_and_draw(&mut self, mbc: &mut Mbc, gw_buffer: &Arc<Mutex<Vec<u8>>>, tcycles: &u64) {

        if !self.ppu_init_complete { return; }
        let buffer_len: u64 = 92160;
        if self.pixel_in_frame * 4 >= buffer_len {
            return;
        }
        let tcycle_budget = tcycles.clone();
        let mut gw_buffer_unlocked = gw_buffer.lock().unwrap();
        let mode_0_h_blank_first_tcycle = 252;
        for _ in 0..tcycle_budget {
            match (self.bg_win_fifo.pop(), self.sprite_fifo.pop()) {
                (Ok(bg_px), Err(_)) => {
                    // push bg_px
                    match self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, bg_px) {
                        Ok(_) => {},
                        Err(PPUEvent::FinishedScanLine) => {
                            self.tcycle_in_scanline = mode_0_h_blank_first_tcycle;
                            break;
                        },
                        _ => { }
                    }
                },
                (Err(_), Ok(sp_px)) => {
                    // push sp_px
                    match self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, sp_px) {
                        Ok(_) => {},
                        Err(PPUEvent::FinishedScanLine) => {
                            self.tcycle_in_scanline = mode_0_h_blank_first_tcycle;
                            break;
                        },
                        _ => { }
                    }
                },
                (Ok(bg_px), Ok(sp_px)) => {
                    if bg_px.bg_priority && bg_px.color != PaletteColor::White {
                        // push_bg_px
                        match self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, bg_px) {
                            Ok(_) => {},
                            Err(PPUEvent::FinishedScanLine) => {
                                self.tcycle_in_scanline = mode_0_h_blank_first_tcycle;
                                break;
                            },
                            _ => { }
                        }

                    } else {
                        // push sp_px
                        match self.push_pixel_and_advance_counter(&mut gw_buffer_unlocked, sp_px) {
                            Ok(_) => {},
                            Err(PPUEvent::FinishedScanLine) => {
                                self.tcycle_in_scanline = mode_0_h_blank_first_tcycle;
                                break;
                            },
                            _ => { }
                        }

                    }
                },
                (Err(_), Err(_)) => {
                    // push none
                    break;
                }
            }
            // let bg_px = self.bg_win_fifo.pop();
            // let sp_px = self.sprite_fifo.pop();
            // if bg_px.is_ok() && sp_px.is_err() {
            //     // push bg_px
            // } else if bg_px.is_err() && sp_px.is_ok() {
            //     // push sp_px
            // } else if bg_px.is_ok() && sp_px.is_ok() {
            //     if bg_px.unwrap().bg_priority && bg_px.unwrap().color != PaletteColor::White {
            //         // push bg pixel
            //     }
            // }

            // if pixels_drew == pixels_to_draw {
            //     return;
            // }
        }



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
            mbc.hw_reg.stat = 0;
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
        let mode_1_v_blank_first_scan_line = 144;
        let current_scanline = mbc.hw_reg.ly;
        let mode_0_h_blank_first_tcycle = 252;
        let mode_3_drawing_first_tcycle = 80;
        let mode_2_oam_scan_last_tcycle = 80;



        //let mode_2_oam_scan_last_cycle: u64 = 80;
        //print!("current scan line is {}\n", current_scanline);
        //print!("current tcycle_in_scanline is {}\n", self.tcycle_in_scanline);
        if current_scanline < mode_1_v_blank_first_scan_line {
            // set the PPU mode when entering a new mode
            if self.tcycle_in_scanline < mode_2_oam_scan_last_tcycle && !self.started_mode_2_in_frame {

                //print!("entering mode_2_oam_scan \n");
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
                self.set_stat_ppu_mode(mbc, PPUMode::OAM_Scan);
                // clear out oam data
                //self.sprites_in_oam_idx = 0;

                self.mode_2_oam_scan(mbc, &tcycle);
                self.started_mode_2_in_frame = true;
            }

            // mode 2 is dot 0-80
            if self.tcycle_in_scanline <  mode_2_oam_scan_last_tcycle {
                // pause for 80 tcycles
                // no need to constantly scan oam again, it's only needed once per scan line
            }


            if self.tcycle_in_scanline >= mode_3_drawing_first_tcycle  && !self.started_mode_3_in_frame {
                // Mode 3 is between 172 and 289 dots, let's call it 172
                //print!("entering mode_3_drawing \n");
                mbc.restrict_vram_access = true;
                //reset fifos
                self.sprite_fifo.data.clear();
                self.bg_win_fifo.data.clear();
                self.fetcher.tcycle_budget = 0;
                //self.fetcher.tile_x_coord = 0;
                // always reset the layer before we start
                self.fetcher.active_layer = Layer::BG;
                self.set_stat_ppu_mode(mbc, PPUMode::Draw);
                self.started_mode_3_in_frame = true;
            }


            if self.tcycle_in_scanline >=  mode_3_drawing_first_tcycle && self.tcycle_in_scanline < mode_0_h_blank_first_tcycle {

                //only draw tiles and bg_map once per mode 3 to reduce utilization
                if !self.drew_tiles_in_mode_3 {
                    self.draw_tiles(mbc, tw);
                    self.draw_bg_map(mbc, bgmw);
                    self.drew_tiles_in_mode_3 = true;
                }
                // todo fix why this is reaching > 255
                let tcycles_res = self.fetcher.tcycle_budget.overflowing_add(tcycle as u8);
                self.fetcher.tcycle_budget = if tcycles_res.1 {
                    255
                } else {
                    tcycles_res.0
                };
                match self.fetcher.active_layer {
                    Layer::BG | Layer::WIN => {
                        //print!("matched BG layer in ppu.tick \n");

                        self.fetcher.handle_bg_win_layer(mbc, &mut self.bg_win_fifo, &mut self.sprite_fifo, &self.sprites, tcycle);
                    },
                    Layer::SPRITE => {
                        //print!("matched sprite layer in ppu.tick \n");
                        self.fetcher.handle_sprite_layer(mbc, &mut self.sprite_fifo, &self.sprites, tcycle);
                    },
                }
                //todo exit mode 3 early if we reach end of scanline

                self.mode_3_mix_pixels_and_draw(mbc, gw, &tcycle);

                self.tcycle_in_mode_3_draw += tcycle;
            }


            if self.tcycle_in_scanline >= mode_0_h_blank_first_tcycle && !self.started_mode_0_in_frame {
                //print!("entering mode_0_h_blank \n");
                mbc.restrict_vram_access = false;
                self.drew_tiles_in_mode_3 = false;
                // Mode 0 is the remainder of the dots left in the scan line (final dot is 456)
                self.set_stat_ppu_mode(mbc, PPUMode::H_Blank);
                self.started_mode_0_in_frame = true;
            }
            if self.tcycle_in_scanline >=  mode_0_h_blank_first_tcycle {

                self.mode_0_h_blank(&tcycle);
            }
        } else {

            self.mode_1_v_blank(mbc, &tcycle);
            // last 10 scan lines are mode 1
            // 4560 dots or 10 scan lines (each scan line is 456 dots)

            let mode_1_v_blank_first_tcycle = 65664;
            if self.tcycle_in_frame >= mode_1_v_blank_first_tcycle && !self.started_mode_1_in_frame {
                //mbc.hw_reg.set_ie_vblank_bit0();
                //print!("entering mode_1_v_blank \n");
                self.fetcher.win_y_pos = 0;
                self.fetcher.tile_y_pos = 0;
                // do not inc tile_x_pos here
                self.set_stat_ppu_mode(mbc, PPUMode::V_Blank);
                self.started_mode_1_in_frame = true;
            }
        }

        // //if all modes  are done cycle back
        if self.started_mode_2_in_frame && self.started_mode_3_in_frame &&
                self.started_mode_0_in_frame && self.started_mode_1_in_frame {
            self.started_mode_2_in_frame = false;
            self.started_mode_3_in_frame = false;
            self.started_mode_0_in_frame = false;
            self.started_mode_1_in_frame = false;
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

        let max_tcycle_in_mode_3_draw: u64 = 23_040;
        //let max_tcycle_in_mode_3_draw: u64 = 24_768;
        //let max_tcycle_in_mode_3_draw: u64 = 41_616;

        let max_tcycle_in_frame = 70_224;
        if self.tcycle_in_mode_3_draw >= max_tcycle_in_mode_3_draw {
            self.tcycle_in_mode_3_draw = 0;
        }

        if self.tcycle_in_frame >= max_tcycle_in_frame {
            //print!("tcycle_in_frame is >= 70224, generating frame \n");
            self.tcycle_in_frame = 0;
            self.tcycle_in_scanline = 0;
            //self.tcycle_in_mode_3_draw = 0;
            PPUEvent::RenderEvent(RenderState::Render)

        } else {
            return PPUEvent::RenderEvent(RenderState::NoRender)
        }
    }
}