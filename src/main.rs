// #![deny(clippy::all)]
#![forbid(unsafe_code)]


use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::gb::graphics::ppu::RenderState;
// use std::thread;
// use std::sync::{Arc, Mutex};

//screen
// const WIDTH: u32 = 256;
// const HEIGHT: u32 = 256;
const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;



mod gb;  

use gb::bios::ColorMode;
use crate::gb::emu::*;
use crate::gb::graphics::lcd::*;



fn main() {

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("REMYUH")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };


    // Create pixel canvas/frame to be modified later
    let mut frame: Pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    // Representation of the object we're drawing
    //let lcd = Lcd::new();

    // setup emu
    let debug = true;

    let mut emu = Emu::new(ColorMode::Gray, debug);
    //let emu_arc = Arc::new(Mutex::new(Box::new(Emu::new(ColorMode::Gray, debug))));
    //let mut emu_arc_clone = Arc::clone(&emu_arc);
    // rom is loaded after bios runs
    //{
        //let mut emu = emu_arc.lock().unwrap();
        emu.load_rom_file(String::from("tetris.gb"));
        emu.load_bios();
    //}


    // thread::spawn(move|| {
    //     let mut emu_thread = emu_arc_clone.lock().unwrap();
    //     loop {
    //         emu_thread.tick();
    //     }
    // });


    // RUN event loop
    let res = event_loop.run(|event, elwt| {
        // Tick
        let render_state = emu.tick(frame.frame_mut());

        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            if render_state == RenderState::render {
                // todo need to modify the code so that emu is not used here, then I can move emu to thread
                //let mut emu = emu_arc.lock().unwrap();
                //lcd.draw(frame.frame_mut(), &mut emu);
            
                frame.render().unwrap();

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
                if let Err(err) = frame.resize_surface(size.width, size.height) {
                    //Lcd::log_error("frame.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }

            // Update internal state and request a redraw
            //lcd.update();
            window.request_redraw();
        }
    });


}
