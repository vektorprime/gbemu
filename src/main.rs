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
use crate::gb::gbwindow::*;



fn main() {

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    let mut input = WinitInputHelper::new();



    // setup emu
    let debug = true;
    let mut emu = Emu::new(ColorMode::Gray, debug);

    // rom is loaded after bios runs
    emu.load_rom_file(String::from("tetris.gb"));
    emu.load_bios();

    let mut game_window = GBWindow::new(WindowType::Game, &event_loop);
    let mut tile_viewer_window = GBWindow::new(WindowType::Tile, &event_loop);

    let tile_viewer_window_id = tile_viewer_window.window.id();
    let game_window_id = game_window.window.id();

    event_loop.run(|event, elwt| {


        let cloned_event = event.clone();
        match cloned_event {
            // Event::AboutToWait => {
            //
            // },
            Event::WindowEvent {window_id, event: window_event} => {
                match window_id {
                    tile_viewer_window_id => {
                        // Draw the current frame
                        //if render_state == RenderState::Render {
                            tile_viewer_window.frame.render().unwrap();
                        //}

                        // Handle input events
                        if input.update(&event) {
                            // Close events
                            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                                elwt.exit();
                                return;
                            }

                            // Resize the window
                            if let Some(size) = input.window_resized() {
                                if let Err(err) = tile_viewer_window.frame.resize_surface(size.width, size.height) {
                                    //Lcd::log_error("frame.resize_surface", err);
                                    elwt.exit();
                                    return;
                                }
                            }

                            tile_viewer_window.window.request_redraw();

                        }
                    },
                    game_window_id => {
                        // Draw the current frame
                        // todo need to somehow get the frame into one tick outside of this loop
                        //let render_state = emu.tick(tile_viewer_window.frame.frame_mut());
                        //if render_state == RenderState::Render {
                            game_window.frame.render().unwrap();
                        //}

                        // Handle input events
                        if input.update(&event) {
                            // Close events
                            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                                elwt.exit();
                                return;
                            }

                            // Resize the window
                            if let Some(size) = input.window_resized() {
                                if let Err(err) = game_window.frame.resize_surface(size.width, size.height) {
                                    //Lcd::log_error("frame.resize_surface", err);
                                    elwt.exit();
                                    return;
                                }
                            }

                            game_window.window.request_redraw();

                        }
                    },
                    _ => {

                    }
                }
            },
            _=> {
                emu.tick(tile_viewer_window.frame.frame_mut());
                tile_viewer_window.window.request_redraw();
                game_window.window.request_redraw();

            },

        }

    }).expect("Unable to run event loop in GBWindow");



}
