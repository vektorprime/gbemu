use crate::gb::mbc::*;
use crate::gb::graphics::fifo::*;
use crate::gb::graphics::ppu::*;
use crate::gb::graphics::sprite::*;
use crate::gb::graphics::pixel::*;
use crate::gb::graphics::tile::{get_tile, Tile, TileType};

const TILES_IN_WIN_ROW: u8 = 20;
const PIXELS_PER_ROW_IN_TILE: u8 = 8;
const ROWS_OF_PIXELS_IN_TILE: u8 = 8;
#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum FetcherError {
    NotEnoughTcycles,
    SwitchedToSpriteLayer,
    NoTileNumFound,
    EndOfScanLine,
    FifoNotEmpty,
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Layer {
    BG,
    WIN,
    SPRITE,
}


pub struct Fetcher {
    pub window_layer_active_in_lcdc: bool,
    pub active_layer: Layer,
    //pub switched_to_window_layer: bool,
    //pub switched_to_sprite_layer: bool,
    pub start_of_rendering: bool,
    pub tile_x_pos: u8,
    pub tile_y_pos: u8,
    pub win_x_pos: u8,
    pub win_y_pos: u8,
    pub dot_in_scanline: u8,
    pub tcycle_budget: u8,
    pub row_in_tile: u8,
    pub current_step: u8,
    pub current_tile_num: u16,
    pub current_tile_low_byte: u8,
    pub current_tile_high_byte: u8,
    pub current_priority: bool,
    pub need_to_resume: bool,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher {
            window_layer_active_in_lcdc: false,
            active_layer: Layer::BG,
            // switched_to_window_layer: false,
            // switched_to_sprite_layer: false,
            start_of_rendering: false,
            tile_x_pos: 0,
            tile_y_pos: 0,
            win_x_pos: 0,
            win_y_pos: 0,
            dot_in_scanline: 0,
            tcycle_budget: 0,
            row_in_tile: 0,
            current_step: 0,
            current_tile_num: 0,
            current_tile_low_byte: 0,
            current_tile_high_byte: 0,
            current_priority: false,
            need_to_resume: false,
        }
    }

    pub fn get_tile_map_address_in_bg_win_step_1(&self, mbc: &Mbc) -> u16 {
        if mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() {
            // are we in a window pixel
             if mbc.hw_reg.ly >= mbc.hw_reg.wy && self.dot_in_scanline >= (mbc.hw_reg.wx - 7) {
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

    pub fn bg_win_step_1_get_tile_num(&mut self, mbc: &Mbc, fifo: &mut Fifo, sprites: &Vec<Sprite>, tcycles: u64) -> Result<usize, FetcherError> {
        self.current_step = 1;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles);
        }
        self.tcycle_budget -= 2;

        let dot = self.dot_in_scanline as u8;
        let dot_tuple = (self.dot_in_scanline as u8).overflowing_add(8);
        let dot_range: u8 = if dot_tuple.1 {
            255
        } else {
            dot_tuple.0
        };

        // check if we need to stop fetching bg_win and switch to the sprite fetcher
        if sprites.iter().any(|&x| x.byte1_x_pos <= dot_range) {
             //print!("switching to sprite layer \n");
             self.active_layer = Layer::SPRITE;
             return Err(FetcherError::SwitchedToSpriteLayer);
         }

        //print!("tcycle_budget is {}\n", self.tcycle_budget);

        let tile_base_add = self.get_tile_map_address_in_bg_win_step_1(mbc);
        // check if we need to switch to window layer
        if mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() && self.active_layer != Layer::WIN {
            // are we in a window pixel
            if mbc.hw_reg.ly == mbc.hw_reg.wy && self.dot_in_scanline >= (mbc.hw_reg.wx - 7) {
                self.active_layer = Layer::WIN;
                //todo add 6 tcycle of delay because fetcher needs to fetch 8 pixels from first win tile
                self.win_x_pos = 0;
                self.win_y_pos = 0;
                self.dot_in_scanline = 0;
            }
        }
        // check if we need to disable switched_to_window_layer every scan line
        if self.active_layer == Layer::WIN {
            if mbc.hw_reg.ly < mbc.hw_reg.wy || !mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() {
                self.active_layer = Layer::BG;
            }
        }
        // get window tile index
        if self.active_layer == Layer::WIN {

            // todo limit tile_index to 384 and return no more drawing needed result
            //print!("getting win tile index in bg_win_step_1_get_tile_num \n");
            //let tile_index = mbc.read(tile_base_add + self.win_x_pos as u16 + (self.win_y_pos  * TILES_IN_WIN_ROW) as u16, OpSource::PPU) as usize;
            let tile_index = mbc.read(tile_base_add  + (32 * (self.win_y_pos / 8)) as u16, OpSource::PPU) as usize;
            //todo get rid of row in tile or use it
            self.row_in_tile += 1;
            //reset to row 0 when we go to a new tile
            if self.row_in_tile == ROWS_OF_PIXELS_IN_TILE { self.row_in_tile = 0; }

            Ok(tile_index)
        } else {
            //print!("getting bg tile index in bg_win_step_1_get_tile_num \n");
            // i doubt window would end mid scan line but let's save this note
            // reset the window_y_pos and window_y_pos since they aren't being used, and we may have stopped mid scan line.
            // get bg

            let x = (((mbc.hw_reg.scx as u16 / 8) + self.tile_x_pos as u16 ) & 0x1F);
            // y is wrong and a really high number which is breaking my drawing
            //let y = 32 * (((mbc.hw_reg.ly + mbc.hw_reg.scy) as u16 & 0xFF) / 8);
            //let y = (mbc.hw_reg.ly + mbc.hw_reg.scy) as u16 & 0xFF;
            let y = (((mbc.hw_reg.ly as u16 + mbc.hw_reg.scy as u16) & 0xFF) / 8) * 32;
            let tile_index = mbc.read(tile_base_add +  x + y, OpSource::PPU) as usize;

            Ok(tile_index)
        }
    }

    pub fn bg_win_step_2_fetch_tile_data_low(&mut self, mbc: &Mbc, tile_num: usize) -> Result<u8, FetcherError> {
        self.current_step = 2;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles)
        }
        self.tcycle_budget -= 2;


        let address: u16 =  if mbc.hw_reg.is_lcdc_bg_win_tile_data_area_bit4_enabled() {
            0x8000
        }
        else {
            0x9000
        };


        let mut tile_byte: u8 = 0;
        if self.active_layer == Layer::WIN {
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = ((tile_num as i8).abs() as u16) * 16;
                    let add = address - neg_offset;
                    return Ok(mbc.read(add + (2 * (self.win_y_pos as u16 % 8)), OpSource::PPU))
                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    return Ok(mbc.read(address + (pos_offset * 16) + (2 * (self.win_y_pos as u16 % 8)), OpSource::PPU))
                }
            }
            else { // handle 0x8000
                let pos_offset = tile_num as u16;
                return Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16, OpSource::PPU))
            }
        }
        else { // fetch bg data
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = ((tile_num as i8).abs() as u16) * 16;
                    let add = address - neg_offset;
                    return Ok(mbc.read(add + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16, OpSource::PPU))
                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    return Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16, OpSource::PPU))
                }
            }
                // todo simplify this when it's confirmed working
            else { // handle 0x8000
                let pos_offset = tile_num as u16;
                return Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16, OpSource::PPU))
            }
        }
    }

    pub fn bg_win_step_3_fetch_tile_data_high(&mut self, mbc: &Mbc, tile_num: usize)  -> Result<u8, FetcherError> {
        self.current_step = 3;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles)
        }
        self.tcycle_budget -= 2;
        // always add 1 here because we want the second byte of data (high byte)
        // fetch window data

        let address: u16 =  if mbc.hw_reg.is_lcdc_bg_win_tile_data_area_bit4_enabled() {
            0x8000
        }
        else {
            0x9000
        };
        if self.active_layer == Layer::WIN {
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = ((tile_num as i8).abs() as u16) * 16;
                    let add = address - neg_offset;
                    return Ok(mbc.read(add + (2 * (self.win_y_pos as u16 % 8)) + 1, OpSource::PPU))

                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    return Ok(mbc.read(address + (pos_offset * 16) + (2 * (self.win_y_pos as u16 % 8)) + 1, OpSource::PPU))
                }
            }
            else { // handle 0x8000
                let pos_offset = tile_num as u16;
                return Ok(mbc.read(address + (pos_offset * 16) + (2 * (self.win_y_pos as u16 % 8)) + 1, OpSource::PPU))
            }
        }
        else { // fetch bg data
            if address == 0x9000 { // handle special case of lcdc bit 4 being 0
                if (tile_num as i8) < 0 { // handle negative
                    let neg_offset = ((tile_num as i8).abs() as u16) * 16;
                    let add = address - neg_offset;
                    return Ok(mbc.read(add + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + 1, OpSource::PPU))
                }
                else { // handle signed positive, easy
                    let pos_offset = tile_num as u16;
                    return Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + 1, OpSource::PPU))
                }
            }
            else { // handle 0x8000
                let pos_offset = tile_num as u16;
                return Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + 1, OpSource::PPU))
            }
        }
    }


    pub fn bg_win_step_4_push_pixels_to_fifo(&mut self, mbc: &Mbc, tile_num: usize, tile_low_byte: u8, tile_high_byte: u8, fifo: &mut Fifo) -> Result<(), FetcherError> {
        self.current_step = 4;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles)
        }
        
        self.tcycle_budget -= 2;

        if self.active_layer == Layer::BG {
            // always return early if the fifo is not empty as per docs
            if !fifo.data.is_empty() { return Err(FetcherError::FifoNotEmpty) }
        }


        // todo if tile is flipped horizontally push lsb first, else push msb first
        let raw_pixels = GBPixel::decode_pixels_from_bytes(tile_low_byte, tile_high_byte);
        for p in raw_pixels {
            fifo.push(GBPixel {
                color: p,
                bg_priority: false,
            });
        }
        if self.active_layer == Layer::WIN {
            self.win_x_pos += 1;
            self.dot_in_scanline += 8;
            if self.dot_in_scanline >= 160 { self.dot_in_scanline = 0; }
            //advance the y and reset 0 in the grid so we always know our position
            if self.win_x_pos == TILES_IN_WIN_ROW {
                self.dot_in_scanline = 0;
                self.win_y_pos += 1;
                self.win_x_pos = 0;
            }
        } else {
            self.tile_x_pos += 1;
            if self.tile_x_pos >= 32 {
                self.tile_x_pos = 0;
            }
            self.dot_in_scanline += 8;
            if self.dot_in_scanline >= 160 {
                self.dot_in_scanline = 0;
            }
        }

        Ok(())
        //if self.dot_in_scanline >= 160 { self.dot_in_scanline = 0; }
    }

    pub fn handle_bg_win_layer(&mut self, mbc: &Mbc, bg_win_fifo: &mut Fifo,  sprite_fifo: &mut Fifo, sprites: &Vec<Sprite>, tcycles: u64) {
       // tcycles handled upstream in ppu tick when matching layer

        if self.need_to_resume && self.current_step > 1 {
            self.need_to_resume = false;
            match self.current_step {
                2 => {
                    let low_byte = match self.bg_win_step_2_fetch_tile_data_low(mbc, self.current_tile_num as usize) {
                        Ok(low_byte) => {
                            self.current_tile_low_byte = low_byte;
                            low_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's low_byte");
                        }
                    };

                    let high_byte = match self.bg_win_step_3_fetch_tile_data_high(mbc, self.current_tile_num as usize) {
                        Ok(high_byte) => {
                            self.current_tile_high_byte = high_byte;
                            high_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's high_byte");
                        }
                    };
                    if self.start_of_rendering {
                        // first fetch of scan line has a delay and must restart
                        self.start_of_rendering = false;
                        return;
                    }
                    self.bg_win_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, low_byte, high_byte, bg_win_fifo);
                },
                3 => {
                    let high_byte = match self.bg_win_step_3_fetch_tile_data_high(mbc, self.current_tile_num as usize) {
                        Ok(high_byte) => {
                            self.current_tile_high_byte = high_byte;
                            high_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's high_byte");
                        }
                    };
                    if self.start_of_rendering {
                        // first fetch of scan line has a delay and must restart
                        self.start_of_rendering = false;
                        return;
                    }
                    match self.bg_win_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, bg_win_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_bg_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }
                },
                4 => {
                    match self.bg_win_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, bg_win_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_bg_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }
                },
                _ => {
                    panic!("Unhandled step in resuming handle_sprite_layer\n");
                },
            }
        }
        else {
            // I set need_to_resume to false here too because I may be resuming step 1 and that will fall into this match
            self.need_to_resume = false;
            match self.bg_win_step_1_get_tile_num(mbc, bg_win_fifo, sprites, tcycles) {
                Ok(tile_num) => {
                    self.current_tile_num = tile_num as u16;
                    let low_byte = match self.bg_win_step_2_fetch_tile_data_low(mbc, self.current_tile_num as usize) {
                        Ok(low_byte) => {
                            self.current_tile_low_byte = low_byte;
                            low_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's low_byte");
                        }
                    };

                    let high_byte = match self.bg_win_step_3_fetch_tile_data_high(mbc, self.current_tile_num as usize) {
                        Ok(high_byte) => {
                            self.current_tile_high_byte = high_byte;
                            high_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's high_byte");
                        }
                    };
                    if self.start_of_rendering {
                        // first fetch of scan line has a delay and must restart
                        self.start_of_rendering = false;
                        return;
                    }
                    match self.bg_win_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, bg_win_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_bg_layer step 4 \n");
                            return;
                        },
                        Err(FetcherError::FifoNotEmpty) => {
                            print!("FIFO not empty in handle_bg_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }                }
                Err(FetcherError::SwitchedToSpriteLayer) => {
                    // first time switching to layer must be handled here as well as upstream in mode_3_draw
                    //print!("switched to sprite layer\n");
                    self.handle_sprite_layer(mbc, sprite_fifo, sprites, tcycles);
                },
                Err(FetcherError::NotEnoughTcycles) => {
                    print!("not enough tcycles, skipping");
                },
                _ => {
                    // todo handle all other errors
                    print!("Unhandled error in handle_bg_win_layer\n");

                }
            }
        }
    }

    pub fn handle_sprite_layer(&mut self, mbc: &Mbc, sprite_fifo: &mut Fifo, sprites: &Vec<Sprite>, tcycles: u64) {
        if self.need_to_resume && self.current_step > 1 {
            self.need_to_resume = false;
            match self.current_step {
                2 => {
                    let low_byte = match self.sprite_step_2_fetch_tile_data_low(mbc, self.current_tile_num as usize) {
                        Ok(low_byte) => {
                            self.current_tile_low_byte = low_byte;
                            low_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_sprite_layer \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's low_byte");
                        }
                    };

                    let high_byte = match self.sprite_step_3_fetch_tile_data_high(mbc, self.current_tile_num as usize) {
                        Ok(high_byte) => {
                            self.current_tile_high_byte = high_byte;
                            high_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's high_byte");
                        }
                    };
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, self.current_priority, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_sprite_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }                    self.active_layer = Layer::BG;
                }
                3 => {
                    let high_byte = match self.sprite_step_3_fetch_tile_data_high(mbc, self.current_tile_num as usize) {
                        Ok(high_byte) => {
                            self.current_tile_high_byte = high_byte;
                            high_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's high_byte");
                        }
                    };
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, self.current_priority, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_sprite_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }
                },
                4 => {
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, self.current_priority, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_sprite_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }                },
                _ => {
                    panic!("Unhandled step in resuming handle_sprite_layer\n");
                },
            }
        }
        else {
            // I set need_to_resume to false here too because I may be resuming step 1 and that will fall into this match
            self.need_to_resume = false;
            let sprite_step_1_result =  self.sprite_step_1_get_tile_num(mbc, sprite_fifo, sprites, tcycles);
            match sprite_step_1_result {
                Ok((tile_num, priority)) => {
                    //print!("inside handle_sprite_layer\n");
                    self.current_tile_num = tile_num as u16;
                    let low_byte = match self.sprite_step_2_fetch_tile_data_low(mbc, self.current_tile_num as usize) {
                        Ok(low_byte) => {
                            self.current_tile_low_byte = low_byte;
                            low_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_sprite_layer step 1 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's low_byte");
                        }
                    };

                    let high_byte = match self.sprite_step_3_fetch_tile_data_high(mbc, self.current_tile_num as usize) {
                        Ok(high_byte) => {
                            self.current_tile_high_byte = high_byte;
                            high_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's high_byte");
                        }
                    };
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, self.current_priority, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            print!("not enough tcycles in handle_sprite_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }
                    self.active_layer = Layer::BG;
                }
                Err(FetcherError::NotEnoughTcycles) => {
                    print!("not enough t cycles, skipping\n");
                },
                Err(FetcherError::NoTileNumFound) => {
                    // should never reach this point because we check for the range of dot + 8 before ever trying in sprite_step_1_get_tile_num
                    //panic!("Could not find tile num for sprite");
                    print!("Could not find tile num for sprite\n");
                    self.active_layer = Layer::BG;
                },
                Err(FetcherError::EndOfScanLine) => {
                    //todo handle end of scanline
                },
                _ => {
                    // todo handle all other errors
                    print!("Unhandled error in handle_sprite_layer\n");
                }
            }
        }




    }


    pub fn sprite_step_1_get_tile_num(&mut self, mbc: &Mbc, fifo: &mut Fifo, sprites: &Vec<Sprite>, tcycles: u64) -> Result<(usize, bool), FetcherError> {
        self.current_step = 1;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles)
        }
        self.tcycle_budget -= 2;

        // sprites are already sorted by X and filtered by Y from mode_2_oam_scan

        let dot = self.dot_in_scanline as u8;
        let dot_tuple = (self.dot_in_scanline as u8).overflowing_add(8);
        let dot_range: u8 = if dot_tuple.1 {
            255
        } else {
            dot_tuple.0
        };

        for x in sprites {
             //if x.byte1_x_pos > dot && x.byte1_x_pos <= dot_range {
            if x.byte1_x_pos <= dot_range {
                 //self.dot_in_scanline += 8;
                 return Ok((x.byte2_tile_num as usize, x.get_byte3_sprite_flags_bit7_priority()));
             }
        }

        if self.dot_in_scanline >= 160 {
            return Err(FetcherError::EndOfScanLine);
        }

        Err(FetcherError::NoTileNumFound)
    }




    pub fn sprite_step_2_fetch_tile_data_low(&mut self, mbc: &Mbc, tile_num: usize) -> Result<u8, FetcherError> {
        self.current_step = 2;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles)
        }
        self.tcycle_budget -= 2;
        // handle 0x8000
        let address: u16 =  0x8000;
        let pos_offset = tile_num as u16;
        return Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16, OpSource::PPU))
    }

    pub fn sprite_step_3_fetch_tile_data_high(&mut self, mbc: &Mbc, tile_num: usize) -> Result<u8, FetcherError>  {
        self.current_step = 3;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles)
        }
        self.tcycle_budget -= 2;

        // todo first time bg fetcher finishes we need to restart to step 1 or delay 12 tcycles
        // handle 0x8000
        let address: u16 =  0x8000;
        let pos_offset = tile_num as u16;
        // always add 1 here because we want the second byte of data (high byte)
        Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + 1, OpSource::PPU))
    }

    pub fn sprite_step_4_push_pixels_to_fifo(&mut self, mbc: &Mbc, tile_num: usize, tile_low_byte: u8, tile_high_byte: u8, priority: bool, fifo: &mut Fifo) -> Result<(), FetcherError>  {
        self.current_step = 4;
        if self.tcycle_budget < 2 {
            self.need_to_resume = true;
            return Err(FetcherError::NotEnoughTcycles)
        }
        self.tcycle_budget -= 2;


        // todo if tile is flipped horizontally push lsb first, else push msb first
        let raw_pixels = GBPixel::decode_pixels_from_bytes(tile_low_byte, tile_high_byte);
        for p in raw_pixels {
            fifo.push(GBPixel {
                color: p,
                // todo handle the bg_priority
                bg_priority: priority,
            });
        }
        Ok(())
    }


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


