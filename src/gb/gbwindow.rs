use std::cmp::PartialEq;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

use std::sync::Arc;

use crate::gb::constants::*;
use crate::gb::graphics::ppu::RenderState;
//screen


pub enum WindowType {
    Tile,
    Game,
}


pub struct GBWindow<'a> {
    pub window: Arc<Window>,
    pub frame: Pixels<'a>,
    pub input: WinitInputHelper,

}


impl<'a> GBWindow<'a> {

    pub fn new(win_type: WindowType, event_loop: &EventLoop<()>, width: u32, height: u32) -> Self {

        //let event_loop = EventLoop::new().unwrap();
        let mut input = WinitInputHelper::new();

        let window_title =  match win_type {
            WindowType::Tile => {
                String::from("REMYUH Tiles")
            },
            WindowType::Game => {
                String::from("REMYUH")
            },
        };
        
        let size = LogicalSize::new(width as f64, height as f64);
        let inner_window = WindowBuilder::new()
                .with_title(window_title)
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap();
        let window = Arc::new(inner_window);
        let window_clone = window.clone();
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window_clone);
        // Create pixel canvas/frame to be modified later
        let mut frame = Pixels::new(width, height, surface_texture).unwrap();

        let input = WinitInputHelper::new();
        
        GBWindow {
            window,
            frame,
            input,
        }

    }
    
    //pub fn tick(&mut self, event_loop: &EventLoop<()>, render_state: &RenderState) {

    //}
}


