use error_iter::ErrorIter as _;
use log::error;

use crate::gb::graphics::palette::*;
use crate::gb::emu::*;
//screen
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;



pub struct Lcd {
    pub x: i16,
    pub y: i16,
    pub velocity_x: i16,
    pub velocity_y: i16,
}



impl Lcd {
    /// Create a new `World` instance that can draw a moving box.
    pub fn new() -> Self {
        Self {
            x: 32,
            y: 16,
            velocity_x: 1,
            velocity_y: 1,
        } 
    } 
 
    /// Update the `World` internal state; bounce the box around the screen.
    pub fn update(&mut self) {
        let i_width: i16 = WIDTH as i16; 
        if self.x >= 0 && self.x <= i_width - BOX_SIZE  {
            self.x += 1;
        } 
    }

    pub fn draw(&self, frame: &mut [u8], emu: &mut Emu) {
        let mut pixels_source: Vec<[u8; 4]> = Vec::new();
        for tile in &emu.ppu.tiles {
            // tile.data is an array of 8 arrays that hold 8 PaletteColor
            for x in 0..8 {
                for y in 0..8 {
                    //put each pixel into a vec so we can move it to the frame later
                    pixels_source.push(tile.data[x][y].get_rgba_code());
                }
            }
        }

        // copy each pixel into the frame
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&pixels_source[i]);
        }
        
    }


    pub fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
        error!("{method_name}() failed: {err}");
        for source in err.sources().skip(1) {
            error!("  Caused by: {source}");
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
}
 