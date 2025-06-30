// #![deny(clippy::all)]
#![forbid(unsafe_code)]


use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{WindowBuilder, WindowId};
use winit_input_helper::WinitInputHelper;

use crate::gb::constants::*;
use crate::gb::graphics::ppu::RenderState;
 use std::thread;
 use std::sync::{Arc, Mutex, mpsc};
 use std::sync::mpsc::{Sender, Receiver};
use std::boxed::Box;
use std::time::{Duration, Instant};
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
    //event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)));
    event_loop.set_control_flow(ControlFlow::Poll);
    /////////////////////////////////////
    // these are for quick debugging
    let skip_render = false;
    let skip_windows = false;
    let debug = true; // true doesn't panic if CPU ticks take longer than a sec
    /////////////////////////////////////

    // setup emu

    let mut emu = Emu::new(ColorMode::Gray, debug);

    // rom is loaded after bios runs
    emu.load_rom_file(String::from("tetris.gb"));
    emu.load_bios();


    if !skip_windows {
        // let game_window = Arc::new(Mutex::new(GBWindow::new(WindowType::Game, &event_loop, 1024, 1024)));
        // let tile_window = Arc::new(Mutex::new(GBWindow::new(WindowType::Tile, &event_loop, 160, 144)));
        let mut game_window = GBWindow::new(WindowType::Game, &event_loop, 1024, 1024);
        let mut tile_window = GBWindow::new(WindowType::Tile, &event_loop, 160, 144);
        let game_window_id = game_window.window.id();
        //print!("game_window_id is {:?}\n", game_window_id);
        let tile_window_id = tile_window.window.id();
        //print!("tile_window_id is {:?}\n", tile_window_id);
        let tile_window_buffer = Arc::new(Mutex::new(vec![0u8; 92_160]));
        let game_window_buffer = Arc::new(Mutex::new(vec![0u8; 4_194_304]));

        let mut render_state = RenderState::Render;
        let mut tile_window_buffer_arc = Arc::clone(&tile_window_buffer);
        let mut tile_window_buffer_arc2 = Arc::clone(&tile_window_buffer);
        let mut game_window_buffer_arc = Arc::clone(&game_window_buffer);
        let mut game_window_buffer_arc2 = Arc::clone(&game_window_buffer);
        thread::spawn(move || {
            loop {
                emu.tick(&tile_window_buffer_arc, &game_window_buffer_arc);
            }
        });
        let mut tw_current_time = Instant::now();
        let mut gw_current_time = Instant::now();
        let one_sec: u64 = 1;
        let mut tw_frames_this_sec: u64 = 0;
        let mut gw_frames_this_sec: u64 = 0;

        let tw_max_fps = 5;
        let gw_max_fps = 60;
        event_loop.run(|event, elwt| {

            let cloned_event = event.clone();
            let mut cloned_window_id = WindowId::clone(&tile_window_id);

            match event {

                Event::WindowEvent { window_id, event: win_event } => {
                    cloned_window_id = window_id.clone();

                    match win_event {

                        WindowEvent::RedrawRequested => {
                            match window_id {
                                tile_window_id => {
                                    //print!("in match win_event redraw requested match window_id for tile_window\n");
                                    // Draw the current frame
                                    if render_state == RenderState::Render && !skip_render {

                                        if tw_current_time.elapsed().as_secs() < one_sec  {
                                            if tw_frames_this_sec < tw_max_fps {
                                                {
                                                    let mut tw_buffer_unlocked = tile_window_buffer_arc2.lock().unwrap();
                                                    let mut tw_pixels = tile_window.frame.frame_mut();
                                                    tw_pixels.copy_from_slice(&tw_buffer_unlocked);
                                                }

                                                tile_window.frame.render().unwrap();
                                            }

                                        }
                                        else {
                                            //print!("sec has elapsed in main tile viewer drawing\n");
                                            tw_current_time = Instant::now();
                                            tw_frames_this_sec = 0;
                                        }

                                    }
                                },
                                game_window_id => {
                                    //print!("in match win_event redraw requested match window_id for game_window\n");
                                    // Draw the current frame

                                    if render_state == RenderState::Render && !skip_render {

                                        if gw_current_time.elapsed().as_secs() < one_sec  {
                                            if gw_frames_this_sec < gw_max_fps {
                                                {
                                                    let mut gw_buffer_unlocked = game_window_buffer_arc2.lock().unwrap();
                                                    let mut gw_pixels = game_window.frame.frame_mut();
                                                    gw_pixels.copy_from_slice(&gw_buffer_unlocked);
                                                }

                                                game_window.frame.render().unwrap();
                                            }
                                        }
                                        else {
                                            //print!("sec has elapsed in main tile viewer drawing\n");
                                            gw_current_time = Instant::now();
                                            gw_frames_this_sec = 0;
                                        }
                                    }

                                },
                                _ => {
                                    panic!("Unable to handle unknown window id in event_loop\n");
                                }
                            }
                        },
                        _ => { }
                    }
                },
                _ => {

                }
            }

            match cloned_window_id {
                tile_window_id => {
                    //print!("in match event_id for tile_window\n");
                    if tile_window.input.update(&cloned_event) {
                        // Close events
                        if tile_window.input.key_pressed(KeyCode::Escape) || tile_window.input.close_requested() {
                            elwt.exit();
                            return;
                        }

                        // Resize the window
                        if let Some(size) = tile_window.input.window_resized() {
                            if let Err(err) = tile_window.frame.resize_surface(size.width, size.height) {
                                elwt.exit();
                                return;
                            }
                        }
                        tile_window.window.request_redraw();

                    }

                },
                game_window_id => {
                    //print!("in match event_id for game_window\n");

                    if game_window.input.update(&cloned_event) {
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

                },
                _ => {
                    panic!("Unable to handle unknown window id in event_loop");
                }
            }
            //game_window.window.request_redraw();
            //tile_window.window.request_redraw();

        // // tile_window.window.request_redraw();
            // // game_window.window.request_redraw();
            // render_state = emu.tick(tile_window.frame.frame_mut(), game_window.frame.frame_mut());
        }).expect("Unable to run event loop in GBWindow");
    } else {
        loop {
            emu.tick_no_window();
        }
    }
}
