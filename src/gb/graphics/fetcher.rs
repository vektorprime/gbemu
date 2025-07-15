use crate::gb::mbc::*;
use crate::gb::graphics::fifo::*;
use crate::gb::graphics::ppu::*;
use crate::gb::graphics::tile::{get_tile, Tile, TileType};

const TILES_IN_WIN_ROW: usize = 20;
const PIXELS_PER_ROW_IN_TILE: usize = 8;
const ROWS_OF_PIXELS_IN_TILE: usize = 8;
#[derive(Debug)]
pub enum FetcherError {
    NotEnoughTcycles,
}

pub struct Fetcher {
    pub window_layer_active_in_lcdc: bool,
    pub switched_to_window_layer: bool,
    pub start_of_rendering: bool,
    pub tile_x_pos: usize,
    pub tile_y_pos: usize,
    pub win_x_pos: usize,
    pub win_y_pos: usize,
    pub dot_in_scanline: usize,
    pub tcycle_budget: u64,
    pub row_in_tile: usize,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher {
            window_layer_active_in_lcdc: false,
            switched_to_window_layer: false,
            start_of_rendering: true,
            tile_x_pos: 0,
            tile_y_pos: 0,
            win_x_pos: 0,
            win_y_pos: 0,
            dot_in_scanline: 0,
            tcycle_budget: 0,
            row_in_tile: 0,
        }
    }

    pub fn get_tile_map_address_in_step_1(&self, mbc: &Mbc) -> u16 {
        if mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() {
            // are we in a window pixel
             if mbc.hw_reg.ly >= mbc.hw_reg.wy && self.dot_in_scanline >= (mbc.hw_reg.wx - 7) as usize {
                 if mbc.hw_reg.is_lcdc_window_tile_map_bit6_enabled() {
                     return 0x9C00
                 } else {
                     if mbc.hw_reg.is_lcdc_bg_tile_map_bit3_enabled() {
                         return 0x9C00
                     } else {
                         return 0x9800
                     }
                 }
             }
            // not in a window, use bg map
            if mbc.hw_reg.is_lcdc_bg_tile_map_bit3_enabled() {
                return 0x9C00
            } else {
                return 0x9800
            }
        } else {
            if mbc.hw_reg.is_lcdc_bg_tile_map_bit3_enabled() {
                return 0x9C00
            } else {
                return 0x9800
            }
        }
    }

    pub fn step_1_get_tile_num(&mut self, mbc: &Mbc, fifo: &mut Fifo, tcycles: u64) -> Result<usize, FetcherError> {
        self.tcycle_budget += tcycles;

        let tile_base_add = self.get_tile_map_address_in_step_1(mbc);
        // check if we need to switch to window layer
        if mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() && !self.switched_to_window_layer {
            // are we in a window pixel
            if mbc.hw_reg.ly == mbc.hw_reg.wy && self.dot_in_scanline >= (mbc.hw_reg.wx - 7) as usize {
                self.switched_to_window_layer = true;
                fifo.bg_win_data.;
                //todo add 6 tcycle of delay because fetcher needs to fetch 8 pixels from first win tile
                self.win_x_pos = 0;
                self.win_y_pos = 0;
                self.dot_in_scanline = 0;
            }
        }
        // check if we need to disable switched_to_window_layer every scan line
        if self.switched_to_window_layer {
            if mbc.hw_reg.ly < mbc.hw_reg.wy || !mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() {
                self.switched_to_window_layer = false;
            }
        }
        // get window tile index
        if self.switched_to_window_layer {
            if self.tcycle_budget < 6 {
                return Err(FetcherError::NotEnoughTcycles)
            }
            self.tcycle_budget -= 6;
            // todo limit tile_index to 384 and return no more drawing needed result
            //let tile_index = mbc.read(tile_base_add + self.win_x_pos as u16 + (self.win_y_pos  * TILES_IN_WIN_ROW) as u16, OpSource::PPU) as usize;
            let tile_index = mbc.read(tile_base_add  + (32 * (self.win_y_pos / 8)) as u16, OpSource::PPU) as usize;
            self.row_in_tile += 1;
            //reset to row 0 when we go to a new tile
            if self.row_in_tile == ROWS_OF_PIXELS_IN_TILE { self.row_in_tile = 0; }
            self.win_x_pos += 1;
            //advance the y and reset 0 in the grid so we always know our position
            if self.win_x_pos == TILES_IN_WIN_ROW {
                self.win_y_pos += 1;
                self.win_x_pos = 0;
            }
            Ok(tile_index)
        } else {
            // i doubt window would end mid scan line but let's save this note
            // reset the window_y_pos and window_y_pos since they aren't being used, and we may have stopped mid scan line.
            let x = (((mbc.hw_reg.scx as u16 / 8) + self.tile_x_pos as u16 ) & 0x1F);
            let y = 32 * (((mbc.hw_reg.ly + mbc.hw_reg.scy) as u16 & 0xFF) / 8);
            let tile_index = mbc.read(tile_base_add +   x + y, OpSource::PPU) as usize;
            self.tile_x_pos += 1;
            Ok(tile_index)
        }
    }

    pub fn step_2_fetch_tile_data_low(&self, mbc: &Mbc, tile_num: usize) -> [u8; 16] {
        // fetch window data

        let address: u16 =  if mbc.hw_reg.is_lcdc_bg_win_tile_data_area_bit4_enabled() {
            0x8000
        }
        else {
            0x9000
        };
            // every 16 bytes is a tile, we'll only fill the first 8 bytes in this step
            let mut tile_bytes: [u8; 16] = [0; 16];
        if self.switched_to_window_layer {
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = (tile_num as i8).abs() as u16;
                    let add = address - neg_offset;
                    for y in 0..16 {
                        tile_bytes[y] = mbc.read(add + (2 * (self.win_y_pos as u16 % 8)) + (y as u16), OpSource::PPU);
                    }
                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    for y in 0..16 {
                        tile_bytes[y] = mbc.read(address + (pos_offset * 16) + (2 * (self.win_y_pos as u16 % 8)) + (y as u16), OpSource::PPU);
                    }
                }
            }
        }
        else { // fetch bg data
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = (tile_num as i8).abs() as u16;
                    let add = address - neg_offset;
                    for y in 0..8 {
                        tile_bytes[y] = mbc.read(add + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + (y as u16), OpSource::PPU);
                    }
                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    for y in 0..8 {
                        tile_bytes[y] = mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + (y as u16), OpSource::PPU);
                    }
                }
            }
        }

        tile_bytes
    }

    pub fn step_3_fetch_tile_data_high(&self, mbc: &Mbc, address: u16, tile_num: usize, tile_bytes: &mut [u8; 16]) {
        // todo first time bg fetecher finishes we need to restart to step 1 or delay 12 tcycles
        // fetch window data
        // every 16 bytes is a tile, we'll fetch the next byte here
        if self.switched_to_window_layer {
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = (tile_num as i8).abs() as u16;
                    let add = address - neg_offset;
                    for y in 0..16 {
                        tile_bytes[y] = mbc.read(add + (2 * (self.win_y_pos as u16 % 8)) + (y as u16), OpSource::PPU);
                    }
                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    for y in 0..16 {
                        tile_bytes[y] = mbc.read(address + (pos_offset * 16) + (2 * (self.win_y_pos as u16 % 8)) + (y as u16), OpSource::PPU);
                    }
                }
            }
        }
        else { // fetch bg data
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = (tile_num as i8).abs() as u16;
                    let add = address - neg_offset;
                    for y in 0..8 {
                        tile_bytes[y] = mbc.read(add + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + (y as u16), OpSource::PPU);
                    }
                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    for y in 0..8 {
                        tile_bytes[y] = mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + (y as u16), OpSource::PPU);
                    }
                }
            }
        }
    }

    pub fn step_4_push_pixels_to_fifo(&self, mbc: &Mbc, tile_num: usize, tile_bytes: &mut [u8; 16], fifo: &mut Fifo) {
        // always return early if the fifo is not empty as per docs
        if !fifo.bg_win_data.is_empty() { return }
        // todo if tile is flipped horizontally push lsb first, else push msb first
        fifo.bg_win_data.extend(tile_bytes.iter());

    }

    // i put too much in this func and it ended up being all steps invovled
    // it turned out good so I wanted to keep it for reference
    // pub fn step_1_get_tile_num(&mut self, mbc: &Mbc, fifo: &mut Fifo, tcycles: u64) -> Result<usize, FetcherError> {
    //     self.tcycle_budget += tcycles;
    //
    //     let tile_base_add = self.get_tile_map_address_in_step_1(mbc);
    //     // check if we need to switch to window layer
    //     if mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() && !self.switched_to_window_layer {
    //         // are we in a window pixel
    //         if mbc.hw_reg.ly == mbc.hw_reg.wy && self.dot_in_scanline >= (mbc.hw_reg.wx - 7) as usize {
    //             self.switched_to_window_layer = true;
    //             fifo.bg_data.clear();
    //             //todo add 6 tcycle of delay because fetcher needs to fetch 8 pixels from first win tile
    //             self.win_x_pos = 0;
    //             self.win_y_pos = 0;
    //             self.dot_in_scanline = 0;
    //         }
    //     }
    //     // check if we need to disable switched_to_window_layer every scan line
    //     if self.switched_to_window_layer {
    //         if mbc.hw_reg.ly < mbc.hw_reg.wy || !mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() {
    //             self.switched_to_window_layer = false;
    //         }
    //     }
    //     // push window pixels to fifo
    //     if self.switched_to_window_layer {
    //         if self.tcycle_budget < 6 {
    //             return Err(FetcherError::NotEnoughTcycles)
    //         }
    //         self.tcycle_budget -= 6;
    //         // todo limit tile_index to 1024 and return no more drawing needed result
    //         let tile_index = mbc.read(tile_base_add + self.win_x_pos as u16 + (self.win_y_pos  * TILES_IN_WIN_ROW) as u16, OpSource::PPU) as usize;
    //         // get the tile via get_tile
    //         let tile = get_tile(mbc, tile_index as u16);
    //         // get the tile row via let tile_row = tile.data[row_in_tile];
    //         let tile_row = tile.data[self.row_in_tile];
    //         // get the tile colors and push_back to vecdeque via a for loop
    //         for &color in tile_row.iter() {
    //             let rgba = color.get_rgba_code();
    //             fifo.win_data.extend(&rgba);
    //             self.dot_in_scanline += 8;
    //         }
    //         self.row_in_tile += 1;
    //         //reset to row 0 when we go to a new tile
    //         if self.row_in_tile == ROWS_OF_PIXELS_IN_TILE { self.row_in_tile = 0; }
    //         self.win_x_pos += 1;
    //         //advance the y and reset 0 in the grid so we always know our position
    //         if self.win_x_pos == TILES_IN_WIN_ROW {
    //             self.win_y_pos += 1;
    //             self.win_x_pos = 0;
    //         }
    //     } else { // push background pixels to fifo
    //
    //     }
    //     Ok (0)
    // }
}

