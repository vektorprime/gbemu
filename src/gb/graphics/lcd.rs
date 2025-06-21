use error_iter::ErrorIter as _;
use log::error;

use crate::gb::graphics::palette::*;
use crate::gb::emu::*;
//screen
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;


#[derive(PartialEq)]
pub enum RenderState {
    render,
    no_render
}

pub struct Lcd {
    // pub temp_frame: [[u8; 4]; 23040]
}


impl Lcd {
    pub fn new() -> Self {
        Self {
        // temp_frame: [[0xFF; 4]; 23040]
        }
    }
 
    // /// Update the `World` internal state; bounce the box around the screen.
    // pub fn update(&mut self) {
    //     let i_width: i16 = WIDTH as i16;
    //     if self.x >= 0 && self.x <= i_width - BOX_SIZE  {
    //         self.x += 1;
    //     }
    // }

    pub fn draw(&self, frame: &mut [u8], emu: &mut Emu) {
        if !emu.ppu.ppu_init_complete { return; }
        //let temp_frame = [[0u8; 4]; 23040];
        let mut pixels_source: Vec<[u8; 4]> = Vec::new();
        // need to iterate over a tile multiple times because now I am drawing the second row on the first row per tile

        let width = 160;
        let tile_width_limit = 128;

        //
        let temp_tiles_per_row :usize = 16;
        let mut temp_tile_count :usize = 0;
        //
        let mut tile_in_grid :usize = 0;
        let rows_per_grid :usize = 8;
        let mut tile_in_row_count = 0;
        let tiles_per_row :usize = 16;
        let rows_per_tile = 8;
        let pixels_per_row = 8;
        let num_of_pixels_to_pad :usize = 32;
        let mut early_break = false;
        let num_of_tiles_in_ppu  = emu.ppu.tiles.len();
        for row_of_tiles_in_grid in 0..rows_per_grid {
            for row in 0..rows_per_tile {
                tile_in_grid = 0;
                for tile in &emu.ppu.tiles {
                    if row_of_tiles_in_grid > 0 {
                        if tile_in_grid < row_of_tiles_in_grid * tiles_per_row {
                            tile_in_grid += 1;
                            continue;
                        }
                    }
                    // pad some bytes because the tiles don't take the whole screen
                    if tile_in_row_count == tiles_per_row {
                        for i in 0..num_of_pixels_to_pad {
                            pixels_source.push([255,255,255,255]);
                        }
                        tile_in_row_count = 0;
                        //tile_in_grid += 1;
                        break;
                    } else {
                        // tile.data is an array of 8 arrays that each hold 8 PaletteColor
                        for pixel in 0..pixels_per_row {
                            //put each pixel into a vec so we can move it to the frame later
                            pixels_source.push(tile.data[row][pixel].get_rgba_code());
                        }
                        tile_in_row_count += 1;
                        //tile_in_grid += 1;
                    }



                }
            }
        }


        // for tile in &emu.ppu.tiles {
        //     // tile.data is an array of 8 arrays that hold 8 PaletteColor
        //     for x in 0..8 {
        //         for y in 0..8 {
        //             //put each pixel into a vec so we can move it to the frame later
        //             pixels_source.push(tile.data[x][y].get_rgba_code());
        //         }
        //     }
        // }

        // copy each pixel into the frame
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if i < pixels_source.len() {
                //let test_slice = [0x00, 0x00, 0xFF, 0xFFu8];
                //pixel.copy_from_slice(&test_slice);
                pixel.copy_from_slice(&pixels_source[i]);
            }
        }

    }


    pub fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
        error!("{method_name}() failed: {err}");
        for source in err.sources().skip(1) {
            error!("  Caused by: {source}");
        }
    }

    // Draw the `World` state to the frame buffer.
    //
    // Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    // pub fn draw(&self, frame: &mut [u8]) {
    //     for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
    //         // the remainder will always get the correct x value (horizontal)
    //         let x: i16 = (i % WIDTH as usize) as i16;
    //         // every WIDTH is a new row (y vertical pixel)
    //         let y = (i / WIDTH as usize) as i16;

    //         // The world object is created and updated in another function, we're just matching it
    //         let obj_is_selected = x >= self.x
    //             && x < self.x + BOX_SIZE
    //             && y >= self.y
    //             && y < self.y + BOX_SIZE;

    //         // modify the pixel (4x u8) if we are drawing our obj
    //         // else draw the background
    //         let rgba = if obj_is_selected {
    //             [0xff, 0x00, 0x00, 0xff]
    //         } else {
    //             [0xff, 0xff, 0xff, 0xff]
    //         };

    //         pixel.copy_from_slice(&rgba);
    //     }
        
    // }



}
 