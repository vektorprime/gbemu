// #![deny(clippy::all)]
#![forbid(unsafe_code)]


use std::env;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{WindowBuilder, WindowId};
use winit_input_helper::WinitInputHelper;

use crate::gb::graphics::ppu::{PPUEvent, RenderState};
 use std::thread;
 use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
//screen

// const WIDTH: u32 = 160;
// const HEIGHT: u32 = 144;




mod gb;  

use gb::bios::ColorMode;
use crate::gb::emu::*;
use crate::gb::gbwindow::*;
use crate::gb::constants::*;


fn main() {
    //env::set_var("RUST_BACKTRACE", "1");
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
    //emu.load_rom_file(String::from("01-special.gb"));
    //emu.load_rom_file(String::from("tetris.gb"));
    //emu.load_rom_file(String::from("dmg-acid2.gb"));
    emu.load_rom_file(String::from("cpu_instrs.gb"));
    //emu.load_rom_file(String::from("addams.gb"));
    //emu.load_rom_file(String::from("boot_regs-A.gb"));
    emu.load_bios();


    if !skip_windows {
       //   let mut bg_map_win = GBWindow::new(WindowType::Game, &event_loop, 1024, 1024);

        let mut tile_win = GBWindow::new(WindowType::Tile, &event_loop, 128, 128);
        let mut bg_map_win = GBWindow::new(WindowType::BGMap, &event_loop, 256, 256);
        let mut game_win = GBWindow::new(WindowType::Game, &event_loop, 160, 144);
        //let mut game_win = GBWindow::new(WindowType::Game, &event_loop, 256, 256);

        let tile_win_id = tile_win.window.id();
        print!("tile_win_id is {:?}\n", tile_win_id);
        let bg_map_win_id = bg_map_win.window.id();
        print!("bg_map_win_id is {:?}\n", bg_map_win_id);
        let game_win_id = game_win.window.id();
        print!("game_win_id is {:?}\n", game_win_id);

        let tile_win_buffer = Arc::new(Mutex::new(vec![0u8; 65_536]));

        let bg_map_win_buffer = Arc::new(Mutex::new(vec![0u8; 262_144]));

        let game_win_buffer = Arc::new(Mutex::new(vec![0u8; 92_160]));
        //let game_win_buffer = Arc::new(Mutex::new(vec![0u8; 262_144]));

        let mut render_state = Arc::new(Mutex::new(PPUEvent::RenderEvent(RenderState::Render)));
        let mut render_state_arc = Arc::clone(&render_state);
        let mut tile_win_buffer_arc = Arc::clone(&tile_win_buffer);
        let mut bg_map_win_buffer_arc = Arc::clone(&bg_map_win_buffer);
        let mut game_win_buffer_arc = Arc::clone(&game_win_buffer);
        thread::spawn(move || {
            loop {
                let mut rs = render_state_arc.lock().unwrap();
                *rs = emu.tick(&tile_win_buffer_arc, &bg_map_win_buffer_arc, &game_win_buffer_arc);
            }
        });
        let mut tw_current_time = Instant::now();
        let mut bgmw_current_time = Instant::now();
        let mut gw_current_time = Instant::now();
        let one_sec: u64 = 1;
        let mut tw_frames_this_sec: u64 = 0;
        let mut bgmw_frames_this_sec: u64 = 0;
        let mut gw_frames_this_sec: u64 = 0;

        let tw_max_fps = 10;
        let bgmw_max_fps = 10;
        let gw_max_fps = 60;

        event_loop.run(|event, elwt| {
            let mut render_state_cloned = PPUEvent::RenderEvent(RenderState::Render);
            {
                //let mut rs = render_state.lock().unwrap();
                //render_state_cloned = *rs;
            }
            let cloned_event = event.clone();
            let mut cloned_window_id = WindowId::clone(&tile_win_id);

            match event {

                Event::WindowEvent { window_id, event: win_event } => {

                    cloned_window_id = window_id.clone();
                    match win_event {
                        WindowEvent::RedrawRequested => {
                            if window_id == tile_win_id {
                                //print!("in match win_event redraw requested match window_id for tile_win\n");
                                // Draw the current frame
                                if render_state_cloned == PPUEvent::RenderEvent(RenderState::Render) && !skip_render {

                                    if tw_current_time.elapsed().as_secs() < one_sec  {
                                        if tw_frames_this_sec < tw_max_fps {
                                            {
                                                let mut tw_buffer_unlocked = tile_win_buffer.lock().unwrap();
                                                let mut tw_pixels = tile_win.frame.frame_mut();
                                                tw_pixels.copy_from_slice(&tw_buffer_unlocked);
                                            }

                                            tile_win.frame.render().unwrap();
                                            tile_win.window.request_redraw();
                                            tw_frames_this_sec += 1;
                                        }

                                    }
                                    else {
                                        //print!("sec has elapsed in main tile viewer drawing\n");
                                        tw_current_time = Instant::now();
                                        tw_frames_this_sec = 0;
                                    }
                                }
                            }
                            if window_id == bg_map_win_id {
                                //print!("in match win_event redraw requested match window_id for bg_map_win\n");
                                // Draw the current frame

                                if render_state_cloned == PPUEvent::RenderEvent(RenderState::Render) && !skip_render {

                                    if bgmw_current_time.elapsed().as_secs() < one_sec  {
                                        if bgmw_frames_this_sec < bgmw_max_fps {
                                            {
                                                let mut bgmw_buffer_unlocked = bg_map_win_buffer.lock().unwrap();
                                                let mut bgmw_pixels = bg_map_win.frame.frame_mut();
                                                bgmw_pixels.copy_from_slice(&bgmw_buffer_unlocked);
                                            }

                                            bg_map_win.frame.render().unwrap();
                                            bg_map_win.window.request_redraw();
                                            bgmw_frames_this_sec += 1;

                                        }
                                    }
                                    else {
                                        //print!("sec has elapsed in main tile viewer drawing\n");
                                        bgmw_current_time = Instant::now();
                                        bgmw_frames_this_sec = 0;
                                    }
                                }
                            }
                            if window_id == game_win_id {
                                //print!("in match win_event redraw requested match window_id for bg_map_win\n");
                                // Draw the current frame

                                if render_state_cloned == PPUEvent::RenderEvent(RenderState::Render) && !skip_render {

                                    if gw_current_time.elapsed().as_secs() < one_sec  {
                                        if gw_frames_this_sec < gw_max_fps {
                                            {
                                                let mut gw_buffer_unlocked = game_win_buffer.lock().unwrap();
                                                let mut gw_pixels = game_win.frame.frame_mut();
                                                gw_pixels.copy_from_slice(&gw_buffer_unlocked);
                                            }

                                            game_win.frame.render().unwrap();
                                            game_win.window.request_redraw();
                                            gw_frames_this_sec += 1;
                                        }
                                    }
                                    else {
                                        //print!("sec has elapsed in main tile viewer drawing\n");
                                        gw_current_time = Instant::now();
                                        gw_frames_this_sec = 0;
                                    }
                                }
                            }


                        },
                        _ => { }
                    }
                },
                _ => {

                }
            }
            if cloned_window_id == tile_win_id {
                //print!("in match event_id for tile_win\n");
                if tile_win.input.update(&cloned_event) {
                    // Close events
                    if tile_win.input.key_pressed(KeyCode::Escape) || tile_win.input.close_requested() {
                        elwt.exit();
                        return;
                    }

                    // Resize the window
                    if let Some(size) = tile_win.input.window_resized() {
                        if let Err(err) = tile_win.frame.resize_surface(size.width, size.height) {
                            elwt.exit();
                            return;
                        }
                    }
                    tile_win.window.request_redraw();
                }

            }

            if cloned_window_id == bg_map_win_id {
                //print!("in match event_id for bg_map_win\n");

                if bg_map_win.input.update(&cloned_event) {
                    // Close events
                    if bg_map_win.input.key_pressed(KeyCode::Escape) || bg_map_win.input.close_requested() {
                        elwt.exit();
                        return;
                    }
                    //0..2 Resize the window
                    if let Some(size) = bg_map_win.input.window_resized() {
                        if let Err(err) = bg_map_win.frame.resize_surface(size.width, size.height) {
                            elwt.exit();
                            return;
                        }
                    }

                    bg_map_win.window.request_redraw();

                }
            }
            if cloned_window_id == game_win_id {
                //print!("in match event_id for bg_map_win\n");

                if game_win.input.update(&cloned_event) {
                    // Close events
                    if game_win.input.key_pressed(KeyCode::Escape) || game_win.input.close_requested() {
                        elwt.exit();
                        return;
                    }
                    //0..2 Resize the window
                    if let Some(size) = game_win.input.window_resized() {
                        if let Err(err) = game_win.frame.resize_surface(size.width, size.height) {
                            elwt.exit();
                            return;
                        }
                    }

                    game_win.window.request_redraw();

                }
            }


            //bg_map_win.window.request_redraw();
            //tile_win.window.request_redraw();

        // // tile_win.window.request_redraw();
            // // bg_map_win.window.request_redraw();
        }).expect("Unable to run event loop in GBWindow");
    } else {
        loop {
            emu.tick_no_window();
        }
    }
}
