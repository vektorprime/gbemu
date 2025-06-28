// #![deny(clippy::all)]
#![forbid(unsafe_code)]


use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::gb::constants::*;
use crate::gb::graphics::ppu::RenderState;
// use std::thread;
// use std::sync::{Arc, Mutex};

//screen

// const WIDTH: u32 = 160;
// const HEIGHT: u32 = 144;




mod gb;  

use gb::bios::ColorMode;
use crate::gb::emu::*;
use crate::gb::graphics::lcd::*;
use crate::gb::gbwindow::*;
use crate::gb::constants::*;


fn main() {

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    
    // setup emu
    let debug = true;
    let mut emu = Emu::new(ColorMode::Gray, debug);

    // rom is loaded after bios runs
    emu.load_rom_file(String::from("tetris.gb"));
    emu.load_bios();

    let mut game_window = GBWindow::new(WindowType::Game, &event_loop, 1024, 1024);
    
    let mut tile_window = GBWindow::new(WindowType::Tile, &event_loop, 160, 144);

    let tile_window_id = tile_window.window.id();
    let game_window_id = game_window.window.id();

    let mut render_state = RenderState::NoRender;
    event_loop.run(|event, elwt| {

        // event is shadowed but th at's ok
        //let cloned_event = event.clone();
        match event {
            // Event::AboutToWait => {
            //
            // },
            Event::WindowEvent {window_id, event: WindowEvent::RedrawRequested} => {
                match window_id {
                    tile_window_id => {
                        // Draw the current frame
                        if render_state == RenderState::Render {
                            tile_window.frame.render().unwrap();
                        }

                    },

                    game_window_id => {
                        // Draw the current frame
                        if render_state == RenderState::Render {
                            game_window.frame.render().unwrap();
                        }

                    },
                    _ => {
                        panic!("Unable to handle unknown window id in event_loop");
                    }
                }
            },
            _=> {
                // process tile window inputs
                // Handle input events
                if tile_window.input.update(&event) {
                    // Close events
                    if tile_window.input.key_pressed(KeyCode::Escape) || tile_window.input.close_requested() {
                        elwt.exit();
                        return;
                    }

                    // Resize the window
                    if let Some(size) = tile_window.input.window_resized() {
                        if let Err(err) = tile_window.frame.resize_surface(size.width, size.height) {
                            //Lcd::log_error("frame.resize_surface", err);
                            elwt.exit();
                            return;
                        }
                    }

                    tile_window.window.request_redraw();

                }
                // process game window inputs
                // Handle input events
                if game_window.input.update(&event) {
                    // Close events
                    if game_window.input.key_pressed(KeyCode::Escape) || game_window.input.close_requested() {
                        elwt.exit();
                        return;
                    }

                    // Resize the window
                    if let Some(size) = game_window.input.window_resized() {
                        if let Err(err) = game_window.frame.resize_surface(size.width, size.height) {
                            //Lcd::log_error("frame.resize_surface", err);
                            elwt.exit();
                            return;
                        }
                    }

                    game_window.window.request_redraw();

                }
                render_state = emu.tick(tile_window.frame.frame_mut(), game_window.frame.frame_mut());
                //tile_window.window.request_redraw();
                //game_window.window.request_redraw();

            },

        }

    }).expect("Unable to run event loop in GBWindow");


}
