use crate::gb::mbc::*;
use crate::gb::graphics::fifo::*;
use crate::gb::graphics::palette::RequestedPalette;
use crate::gb::graphics::ppu::*;
use crate::gb::graphics::sprite::*;
use crate::gb::graphics::pixel::*;
use crate::gb::graphics::tile::{get_tile, Tile, TileType};

// const TILES_IN_WIN_ROW: u8 = 20;
// const PIXELS_PER_ROW_IN_TILE: u8 = 8;
// const ROWS_OF_PIXELS_IN_TILE: u8 = 8;

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum FetcherError {
    NotEnoughTcycles,
    SwitchedToSpriteLayer,
    NoSpriteFound,
    EndOfScanLine,
    FifoNotEmpty,
    FifoFull,
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Layer {
    BG,
    WIN,
    SPRITE,
}


pub struct Fetcher {
    pub window_layer_active_in_frame: bool,
    pub window_layer_active_in_scanline: bool,
    pub active_layer: Layer,
    //pub switched_to_window_layer: bool,
    //pub switched_to_sprite_layer: bool,
    pub start_of_rendering: bool,
    pub current_bg_tile_x_pos: u16,
    pub current_sprite_tile_x_pos: u16,
    pub current_tile_y_pos: u8,
    pub win_x_pos: u8,
    pub win_y_pos: u8,
    //pub dot_in_scanline: u8,
    pub tcycle_budget: u8,
    //pub row_in_tile: u8,
    pub bg_layer_current_step: u8,
    pub sprite_layer_current_step: u8,
    pub current_tile_num: u16,
    pub current_tile_low_byte: u8,
    pub current_tile_high_byte: u8,
    //pub current_priority: bool,
    pub bg_layer_need_to_resume: bool,
    pub sprite_layer_need_to_resume: bool,
    pub pixels_to_mark_skipped: u8,
    pub remaining_bg_pixels_before_sprite: u8,
    //pub fetcher_end_of_scanline: bool,
    pub finished_sprites_in_scanline: bool,
    pub scan_for_sprites_in_last_fetch: bool,
    current_sprite: Option<Sprite>,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher {
            window_layer_active_in_frame: false,
            window_layer_active_in_scanline: false,
            active_layer: Layer::BG,
            // switched_to_window_layer: false,
            // switched_to_sprite_layer: false,
            start_of_rendering: false,
            current_bg_tile_x_pos: 0,
            current_sprite_tile_x_pos: 0,
            current_tile_y_pos: 0,
            win_x_pos: 0,
            win_y_pos: 0,
            //dot_in_scanline: 0,
            tcycle_budget: 0,
            //row_in_tile: 0,
            bg_layer_current_step: 0,
            sprite_layer_current_step: 0,
            current_tile_num: 0,
            current_tile_low_byte: 0,
            current_tile_high_byte: 0,
            //current_priority: false,
            bg_layer_need_to_resume: false,
            sprite_layer_need_to_resume: false,
            pixels_to_mark_skipped: 0,
            remaining_bg_pixels_before_sprite: 0,
            //fetcher_end_of_scanline: false,
            finished_sprites_in_scanline: false,
            scan_for_sprites_in_last_fetch: false,
            current_sprite: None,
        }
    }

    pub fn get_tile_map_address_in_bg_win_step_1(&self, mbc: &Mbc) -> u16 {
        if mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() && mbc.hw_reg.is_lcdc_bg_and_win_enable_bit0_enabled() {
            // are we in a window pixel
            if mbc.hw_reg.ly >= mbc.hw_reg.wy && (self.current_bg_tile_x_pos * 8) as i16 >= (mbc.hw_reg.wx as i16 - 7) {
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
                0x9C00
            } else {
                0x9800
            }
        } else {
            if mbc.hw_reg.is_lcdc_bg_tile_map_bit3_enabled() {
                0x9C00
            } else {
                0x9800
            }
        }
    }

    pub fn bg_win_step_1_get_tile_num(&mut self, mbc: &Mbc, fifo: &mut Fifo, sprites: &mut Vec<Sprite>, tcycles: u64) -> Result<usize, FetcherError> {
        self.bg_layer_current_step = 1;
        // todo re-enable cycle budget to make it cycle accurate, but it skews the image
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles);
        // }
        // self.tcycle_budget -= 2;

        //for debug
        // if mbc.hw_reg.ly == 0x70 {
        //     print!("ly is 0x70\n");
        // }

       // let inital_layer = self.active_layer;

        self.remaining_bg_pixels_before_sprite = 8;
        if !self.finished_sprites_in_scanline {
            let current_dot = self.current_bg_tile_x_pos as u8 * 8;
            let dot_range = current_dot + 7;
            // check if we need to stop fetching bg_win and switch to the sprite fetcher
            if !sprites.is_empty() {
                for (i, x) in sprites.iter().enumerate() {
                    let sprite_x_pos = x.byte1_x_pos - 8;
                    if sprite_x_pos  >= current_dot && sprite_x_pos <= dot_range {
                        self.remaining_bg_pixels_before_sprite = sprite_x_pos - current_dot;
                        //print!("self.remaining_bg_pixels_before_switching_layer is {}\n", self.remaining_bg_pixels_before_sprite);
                    }
                }
            }
        }

        //print!("tcycle_budget is {}\n", self.tcycle_budget);

        let mut tile_base_add = self.get_tile_map_address_in_bg_win_step_1(mbc);
        // check if we need to switch to window layer
        if mbc.hw_reg.is_lcdc_window_enable_bit5_enabled() && mbc.hw_reg.is_lcdc_bg_and_win_enable_bit0_enabled()  {
            // are we in a window pixel
            // doesn't need saturating sub because I am using i16
            if mbc.hw_reg.ly >= mbc.hw_reg.wy && (self.current_bg_tile_x_pos * 8) as i16 >= (mbc.hw_reg.wx as i16 - 7) {
                // print!("LY is {}, and WY is {}\n",mbc.hw_reg.ly, mbc.hw_reg.wy);
                // print!("WX is {} \n", (mbc.hw_reg.wx).wrapping_sub(7));
                self.active_layer = Layer::WIN;

                if !self.window_layer_active_in_scanline && self.window_layer_active_in_frame {
                    self.window_layer_active_in_scanline = true;
                    self.win_y_pos += 1;
                    //self.win_x_pos = 0;
                }
                //print!("switching to WIN layer\n");
                if !self.window_layer_active_in_frame {
                    self.window_layer_active_in_frame = true;
                    self.win_x_pos = 0;
                    self.win_y_pos = 0;
                }
            } else {
                self.active_layer = Layer::BG;
            }
        } else {
            self.active_layer = Layer::BG;
        }


        // get window tile index
        let tile_index = if self.active_layer == Layer::WIN {
            //print!("getting win tile index in bg_win_step_1_get_tile_num \n");
            let win_x = self.win_x_pos;
            //let win_y: u16 = (mbc.hw_reg.ly as u16 - mbc.hw_reg.wy as u16) / 8 * 32;
            let win_y: u16 = (self.win_y_pos as u16 / 8 ) * 32;
            mbc.read(tile_base_add + win_x as u16 + win_y as u16, OpSource::PPU) as usize
        } else {
            // bg index
            //print!("getting bg tile index in bg_win_step_1_get_tile_num \n");
            //print!("scx is {} \n", mbc.hw_reg.scx);
            let x = (((mbc.hw_reg.scx as u16 / 8) + self.current_bg_tile_x_pos as u16 ) & 0x1F);
            // handles pixel row
            let y = (((mbc.hw_reg.ly as u16 + mbc.hw_reg.scy as u16) & 0xFF) / 8) * 32;
            mbc.read(tile_base_add + x + y, OpSource::PPU) as usize
        };
        self.bg_layer_current_step = 2;
        Ok(tile_index)
    }

    pub fn bg_win_step_2_fetch_tile_data_low(&mut self, mbc: &Mbc, tile_num: usize, ) -> Result<u8, FetcherError> {
        self.bg_layer_current_step = 2;
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles);
        // }
        // self.tcycle_budget -= 2;

        // Determine which tile data area
        let use_unsigned = mbc.hw_reg.is_lcdc_bg_win_tile_data_area_bit4_enabled();
        let row_offset: u16 = if self.active_layer == Layer::WIN {
            (2 * (mbc.hw_reg.ly - mbc.hw_reg.wy)  % 8) as u16
        } else {
            let fine_y = ((mbc.hw_reg.ly as u16 + mbc.hw_reg.scy as u16) & 0xFF) % 8;
            2 * fine_y
        };

        let addr = if use_unsigned {
            // Unsigned addressing, base 0x8000
            0x8000u16 + (tile_num as u16 * 16) + row_offset
        } else {
            // Signed addressing, base 0x9000
            let tile_num_signed = tile_num as i8 as i16;
            (0x9000i32 + (tile_num_signed as i32) * 16 + row_offset as i32) as u16
        };

        self.bg_layer_current_step = 3;
        Ok(mbc.read(addr, OpSource::PPU))
    }


    pub fn bg_win_step_3_fetch_tile_data_high(&mut self, mbc: &Mbc, tile_num: usize) -> Result<u8, FetcherError> {
        self.bg_layer_current_step = 3;
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles);
        // }
        // self.tcycle_budget -= 2;

        let use_unsigned = mbc.hw_reg.is_lcdc_bg_win_tile_data_area_bit4_enabled();
        let row_offset: u16 = if self.active_layer == Layer::WIN {
            (2 * (mbc.hw_reg.ly - mbc.hw_reg.wy)  % 8) as u16
        } else {
            let fine_y = ((mbc.hw_reg.ly as u16 + mbc.hw_reg.scy as u16) & 0xFF) % 8;
            2 * fine_y
        };

        let addr = if use_unsigned {
            0x8000u16 + (tile_num as u16 * 16) + row_offset + 1
        } else {
            let signed_index = tile_num as i8 as i16;
            (0x9000i32 + (signed_index as i32) * 16 + row_offset as i32 + 1) as u16
        };

        self.bg_layer_current_step = 4;
        Ok(mbc.read(addr, OpSource::PPU))
    }


    pub fn bg_win_step_4_push_pixels_to_fifo(&mut self, mbc: &Mbc, tile_num: usize, tile_low_byte: u8, tile_high_byte: u8, fifo: &mut Fifo) -> Result<(), FetcherError> {
        self.bg_layer_current_step = 4;
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles)
        // }
        //
        // self.tcycle_budget -= 2;


            if !fifo.data.is_empty() {
                print!("BG FIFO not empty, can't push\n");
                return Err(FetcherError::FifoNotEmpty);
            }

        let raw_pixels = GBPixel::decode_pixels_from_bytes(mbc, RequestedPalette::BG, tile_low_byte, tile_high_byte, false);
        for p in raw_pixels {


            let skip = if self.pixels_to_mark_skipped > 0 {
                self.pixels_to_mark_skipped -= 1;
                true
            } else {
                false
            };



            match fifo.push(
                GBPixel {
                color: p,
                bg_priority: false,
                skip, }) {
                Ok(_) => {},
                Err(FifoOpError::FifoFull) => {
                    // todo handle this error
                    print!("Fifo length exceeded, cannot push pixels\n");
                },
                Err(_) => {
                    panic!("Unhandled error in fifo push\n");
                }
            }

            // // this is used to skip pushing pixels when the sprite starts at an offset that's not divisible by 8
            if self.remaining_bg_pixels_before_sprite > 0 {
                self.remaining_bg_pixels_before_sprite -= 1;
            }
            else {
                self.scan_for_sprites_in_last_fetch = true;
                break;
            }

        }


        if self.active_layer == Layer::WIN {
            self.win_x_pos += 1;
        }


        self.current_bg_tile_x_pos += 1;
        // needed because it's used in the scan line calc in func get_tile_map_address_in_bg_win_step_1
        if self.current_bg_tile_x_pos == 20 {
            self.current_bg_tile_x_pos = 0;
            self.win_x_pos = 0;
        }


        // self.current_bg_tile_x_pos += 1;
        // // needed because it's used in the scan line calc in func get_tile_map_address_in_bg_win_step_1
        // if self.current_bg_tile_x_pos == 20 {
        //     self.current_bg_tile_x_pos = 0;
        //     //self.win_x_pos = 0;
        //
        // } else {
        //     if self.active_layer == Layer::WIN {
        //         self.win_x_pos += 1;
        //         if self.win_x_pos == 32 {
        //             self.win_x_pos = 0;
        //         }
        //     }
        // }

        self.bg_layer_current_step = 1;
        Ok(())
    }

    pub fn handle_bg_win_layer(&mut self, mbc: &Mbc, bg_win_fifo: &mut Fifo, sprites: &mut Vec<Sprite>, tcycles: u64)  {
       // tcycles handled upstream in ppu tick when matching layer

        if self.bg_layer_need_to_resume && self.bg_layer_current_step > 1 {
            self.bg_layer_need_to_resume = false;
            match self.bg_layer_current_step {
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
                    // this is skewing the image do NOT enable
                    // if self.start_of_rendering {
                    //     // first fetch of scan line has a delay and must restart
                    //     self.start_of_rendering = false;
                    //     return;
                    // }
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
                    // this is skewing the image do NOT enable
                    // if self.start_of_rendering {
                    //     // first fetch of scan line has a delay and must restart
                    //     self.start_of_rendering = false;
                    //     return;
                    // }
                    match self.bg_win_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, bg_win_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            // print!("not enough tcycles in handle_bg_layer step 4 \n");
                            return;
                        },
                        Err(FetcherError::FifoNotEmpty) => {
                            // print!("FIFO not empty in bg_win_step_4_push_pixels_to_fifo \n");
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
                            // print!("not enough tcycles in handle_bg_layer step 4 \n");
                            return;
                        },
                        Err(FetcherError::FifoNotEmpty) => {
                            // print!("FIFO not empty in bg_win_step_4_push_pixels_to_fifo \n");
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
            self.bg_layer_need_to_resume = false;
            match self.bg_win_step_1_get_tile_num(mbc, bg_win_fifo, sprites,  tcycles) {
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
                    // this is skewing the image do NOT enable
                    // if self.start_of_rendering {
                    //     // first fetch of scan line has a delay and must restart
                    //     self.start_of_rendering = false;
                    //     return;
                    // }
                    match self.bg_win_step_4_push_pixels_to_fifo(mbc, self.current_tile_num as usize, self.current_tile_low_byte, self.current_tile_high_byte, bg_win_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            // print!("not enough tcycles in handle_bg_layer step 4 \n");
                            return;
                        },
                        Err(FetcherError::FifoNotEmpty) => {
                            // print!("FIFO not empty in handle_bg_layer step 4 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }                }
                // Err(FetcherError::SwitchedToSpriteLayer) => {
                //     // first time switching to layer must be handled here as well as upstream in mode_3_draw
                //     //print!("switched to sprite layer\n");
                //     self.handle_sprite_layer(mbc, sprite_fifo, sprites, tcycles);
                // },
                Err(FetcherError::NotEnoughTcycles) => {
                    print!("not enough tcycles, skipping");
                },
                _ => {
                    // todo handle all other errors
                    panic!("Unhandled error in handle_bg_win_layer\n");

                }
            }
        }
    }




    pub fn sprite_step_1_get_tile_num(&mut self, mbc: &Mbc, fifo: &mut Fifo, sprites: &mut Vec<Sprite>, tcycles: u64) -> Result<Sprite, FetcherError> {
        self.bg_layer_current_step = 1;
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles)
        // }
        // self.tcycle_budget -= 2;

        // sprites are already sorted by X and filtered by Y from mode_2_oam_scan
        // do NOT -8 this

        let current_dot = (self.current_sprite_tile_x_pos as u8) * 8;
        let dot_range = current_dot + 7;
        // let mut sprite_num: usize = 0;
        // let mut sprite_priority = false;
        let mut idx_to_remove = 0;
        let mut found_sprite = false;

        if !sprites.is_empty() {
        // check previous fetch for sprites that started at an offset not divisible by 8
            for (i, x) in sprites.iter().enumerate() {
                let sprite_x_pos = x.byte1_x_pos - 8;
                if sprite_x_pos  >= current_dot && sprite_x_pos <= dot_range {
                    //sprite_num = x.byte2_tile_num as usize;
                    //sprite_priority = x.get_byte3_sprite_flags_bit7_priority();
                    idx_to_remove = i;
                    found_sprite = true;
                    self.scan_for_sprites_in_last_fetch = false;
                    break;
                }
            }
        } else {
            self.finished_sprites_in_scanline = true;
            //print!("setting finished_sprites_in_scanline to true \n");
        }

        let sprite = if found_sprite {
                sprites.remove(idx_to_remove)
            }
            else {
                return Err(FetcherError::NoSpriteFound)
        };


        self.bg_layer_current_step = 2;
        Ok(sprite)


    }




    // pub fn sprite_step_2_fetch_tile_data_low(&mut self, mbc: &Mbc, tile_num: usize) -> Result<u8, FetcherError> {
    //     self.current_step = 2;
    //     if self.tcycle_budget < 2 {
    //         self.need_to_resume = true;
    //         return Err(FetcherError::NotEnoughTcycles)
    //     }
    //     self.tcycle_budget -= 2;
    //     // handle 0x8000
    //     let address: u16 =  0x8000;
    //     let pos_offset = tile_num as u16;
    //     return Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16, OpSource::PPU))
    // }
    //
    // pub fn sprite_step_3_fetch_tile_data_high(&mut self, mbc: &Mbc, tile_num: usize) -> Result<u8, FetcherError>  {
    //     self.current_step = 3;
    //     if self.tcycle_budget < 2 {
    //         self.need_to_resume = true;
    //         return Err(FetcherError::NotEnoughTcycles)
    //     }
    //     self.tcycle_budget -= 2;
    //
    //     // todo first time bg fetcher finishes we need to restart to step 1 or delay 12 tcycles
    //     // handle 0x8000
    //     let address: u16 =  0x8000;
    //     let pos_offset = tile_num as u16;
    //     // always add 1 here because we want the second byte of data (high byte)
    //     Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + 1, OpSource::PPU))
    // }

    pub fn sprite_step_2_fetch_tile_data_low(&mut self, mbc: &Mbc) -> Result<u8, FetcherError> {
        self.sprite_layer_current_step = 2;
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles)
        // }
        // self.tcycle_budget -= 2;
        // handle 0x8000
        let address: u16 =  0x8000;

        let fine_y = ((mbc.hw_reg.ly as u16 + mbc.hw_reg.scy as u16) & 0xFF) % 8;
        let row_offset = 2 * fine_y;
        self.sprite_layer_current_step = 3;

        //let x_flip = self.current_sprite.as_ref().unwrap().get_byte3_sprite_flags_bit5_xflip();
        let y_flip = self.current_sprite.as_ref().unwrap().get_byte3_sprite_flags_bit6_yflip();

        if y_flip {
            return Ok(mbc.read(address + (self.current_sprite.as_ref().unwrap().byte2_tile_num as u16 * 16) + 14 - row_offset, OpSource::PPU))
        } else {
            return Ok(mbc.read(address + (self.current_sprite.as_ref().unwrap().byte2_tile_num as u16 * 16) + row_offset, OpSource::PPU))
        }
    }

    // pub fn sprite_step_3_fetch_tile_data_high(&mut self, mbc: &Mbc, tile_num: usize) -> Result<u8, FetcherError>  {
    //     self.current_step = 3;
    //     if self.tcycle_budget < 2 {
    //         self.need_to_resume = true;
    //         return Err(FetcherError::NotEnoughTcycles)
    //     }
    //     self.tcycle_budget -= 2;
    //
    //     // todo first time bg fetcher finishes we need to restart to step 1 or delay 12 tcycles
    //     // handle 0x8000
    //     let address: u16 =  0x8000;
    //     let pos_offset = tile_num as u16;
    //     // always add 1 here because we want the second byte of data (high byte)
    //     Ok(mbc.read(address + (pos_offset * 16) + (2 * ((mbc.hw_reg.ly + mbc.hw_reg.scy) % 8)) as u16 + 1, OpSource::PPU))
    // }

    pub fn sprite_step_3_fetch_tile_data_high(&mut self, mbc: &Mbc) -> Result<u8, FetcherError>  {
        self.sprite_layer_current_step = 3;
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles)
        // }
        // self.tcycle_budget -= 2;
        // handle 0x8000
        let address: u16 =  0x8000;

        let fine_y = ((mbc.hw_reg.ly as u16 + mbc.hw_reg.scy as u16) & 0xFF) % 8;
        let row_offset = 2 * fine_y;
        self.sprite_layer_current_step = 4;

        //let x_flip = self.current_sprite.as_ref().unwrap().get_byte3_sprite_flags_bit5_xflip();
        let y_flip = self.current_sprite.as_ref().unwrap().get_byte3_sprite_flags_bit6_yflip();

        if y_flip {
            return Ok(mbc.read(address + (self.current_sprite.as_ref().unwrap().byte2_tile_num as u16 * 16) + 14 + 1 - row_offset, OpSource::PPU))
        } else {
            return Ok(mbc.read(address + (self.current_sprite.as_ref().unwrap().byte2_tile_num as u16 * 16) + 1 + row_offset, OpSource::PPU))
        }
    }



    pub fn sprite_step_4_push_pixels_to_fifo(&mut self, mbc: &Mbc, fifo: &mut Fifo) -> Result<(), FetcherError>  {
        self.sprite_layer_current_step = 4;
        // if self.tcycle_budget < 2 {
        //     self.need_to_resume = true;
        //     return Err(FetcherError::NotEnoughTcycles)
        // }
        // self.tcycle_budget -= 2;

        // todo re-enable after fixing draw, re-analyze to determine if needed
        // let pixels_to_skip =  fifo.data.len();
        // if pixels_to_skip > 0 {
        //     // self.pixels_to_mark_skipped += pixels_to_skip as u8;
        //     // fifo.data.clear();
        //     return Err(FetcherError::FifoFull);
        // }


        //todo handle flipping horizontally and vertically
        //
        // To flip y we should read from right to left (MSB), need the later bytes
        // To flip x We should read the second byte first then the first (each 2 bytes is a line)

        let x_flip = self.current_sprite.as_ref().unwrap().get_byte3_sprite_flags_bit5_xflip();
        let y_flip = self.current_sprite.as_ref().unwrap().get_byte3_sprite_flags_bit6_yflip();

        let raw_pixels = GBPixel::decode_pixels_from_bytes(mbc, RequestedPalette::Sprite, self.current_tile_low_byte, self.current_tile_high_byte, x_flip);


        for p in raw_pixels {

            let skip = if self.pixels_to_mark_skipped > 0 {
                self.pixels_to_mark_skipped -= 1;
                true
            } else {
                false
            };

            match fifo.push(
                GBPixel {
                    color: p,
                    bg_priority: false,
                    skip, }
            ) {
                Ok(()) => {
                    //print!("");
                },
                Err(FifoOpError::FifoFull) => {
                    // todo handle this error
                    return Err(FetcherError::FifoFull);
                    print!("FIFO length exceeded, cannot push pixels\n");
                },
                Err(_) => {
                    panic!("Unhandled error in fifo push\n");
                }
            }
        }

        //  // these are still used in step 1 of sprite
          self.current_sprite_tile_x_pos += 1;
         // needed because it's used in the scan line calc in func get_tile_map_address_in_bg_win_step_1
        if self.current_sprite_tile_x_pos == 20 {
            self.current_sprite_tile_x_pos = 0;
        }
        self.sprite_layer_current_step = 1;
        Ok(())
    }
    pub fn handle_sprite_layer(&mut self, mbc: &Mbc, sprite_fifo: &mut Fifo, sprites: &mut Vec<Sprite>, tcycles: u64) {
        if self.sprite_layer_need_to_resume && self.sprite_layer_current_step > 1 {
            self.sprite_layer_need_to_resume = false;
            match self.sprite_layer_current_step {
                2 => {
                    let low_byte = match self.sprite_step_2_fetch_tile_data_low(mbc) {
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

                    let high_byte = match self.sprite_step_3_fetch_tile_data_high(mbc) {
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
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            // print!("not enough tcycles in handle_sprite_layer step 4 \n");
                            return;
                        },
                        Err(FetcherError::FifoFull) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }
                    self.active_layer = Layer::BG;
                }
                3 => {
                    let high_byte = match self.sprite_step_3_fetch_tile_data_high(mbc) {
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
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            // print!("not enough tcycles in handle_sprite_layer step 4 \n");
                            return;
                        },
                        Err(FetcherError::FifoFull) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }
                },
                4 => {
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            // print!("not enough tcycles in handle_sprite_layer step 4 \n");
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
            self.sprite_layer_need_to_resume = false;
            let sprite_step_1_result =  self.sprite_step_1_get_tile_num(mbc, sprite_fifo, sprites, tcycles);
            match sprite_step_1_result {
                Ok(sprite) => {
                    //print!("inside handle_sprite_layer\n");
                    self.current_sprite = Some(sprite);
                    match self.sprite_step_2_fetch_tile_data_low(mbc) {
                        Ok(low_byte) => {
                            self.current_tile_low_byte = low_byte;
                            low_byte
                        },
                        Err(FetcherError::NotEnoughTcycles) => {
                            // print!("not enough tcycles in handle_sprite_layer step 1 \n");
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's low_byte");
                        }
                    };

                    match self.sprite_step_3_fetch_tile_data_high(mbc) {
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
                    match self.sprite_step_4_push_pixels_to_fifo(mbc, sprite_fifo) {
                        Ok(_) => {},
                        Err(FetcherError::NotEnoughTcycles) => {
                            // print!("not enough tcycles in handle_sprite_layer step 4 \n");
                            return;
                        },
                        Err(FetcherError::FifoFull) => {
                            return;
                        },
                        _ => {
                            panic!("unknown error in handle_bg_win_layer's step 4");
                        }
                    }
                    //self.active_layer = Layer::BG;
                }
                Err(FetcherError::NotEnoughTcycles) => {
                    print!("not enough t cycles, skipping\n");
                },
                Err(FetcherError::NoSpriteFound) => {
                    //panic!("Could not find tile num for sprite");
                    //print!("Could not find tile num for sprite\n");
                    // todo, I believe this is the issue where the window is not drawing

                    self.active_layer = Layer::BG;

                    // if self.active_layer == Layer::SPRITE {
                    //     self.active_layer = Layer::BG;
                    // }

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

}



    // i put too much in this func and it ended up being all steps involved
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


