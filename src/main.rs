// #![deny(clippy::all)]
#![forbid(unsafe_code)]


use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

//screen
const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const BOX_SIZE: i16 = 64;

mod gb;  

use gb::bios::ColorMode;
use crate::gb::emu::*;
use crate::gb::graphics::lcd::*;

fn main() -> Result<(), Error>  {

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
 
    // Create pixel canvas/frame to be modified later
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    // Representation of the object we're drawing
    

    //////////////////////
    // setup emu
    let debug = true;
    let mut emu = Emu::new(ColorMode::Gray, debug);
    // rom is loaded after bios runs
    emu.load_rom_file(String::from("tetris.gb"));
    emu.load_bios();
    let mut lcd = Lcd::new();
    emu.init_ppu();
    //////////////////////
    // RUN event loop
    let res = event_loop.run(|event, elwt| {
        //////////////////////
        // todo
        // finish cpu code
        // finish drawing code
        emu.tick();
        //////////////////////
        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            lcd.draw(pixels.frame_mut(), &mut emu);
            if let Err(err) = pixels.render() {
                Lcd::log_error("pixels.render", err);
                elwt.exit();
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    Lcd::log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }

            // Update internal state and request a redraw
            lcd.update();
            window.request_redraw();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))

    

}
