

use winit::keyboard::KeyCode;
use crate::gb::mbc::Mbc;

use winit::event::ElementState;

pub struct Joypad {
    // pub a_right: bool,
    // pub b_left: bool,
    // pub select_up: bool,
    // pub start_down: bool,

    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,

    pub select_dpad: bool,
    pub select_buttons: bool,

    pub is_reg_pending_update_from_joypad: bool,
    pub is_pending_joypad_interrupt_trigger: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            // a_right: false,
            // b_left: false,
            // select_up: false,
            // start_down: false,
            left: false,
            right: false,
            up: false,
            down: false,
            a: false,
            b: false,
            select: false,
            start: false,
            select_dpad: false,
            select_buttons: false,
            is_reg_pending_update_from_joypad: false,
            is_pending_joypad_interrupt_trigger: false,
        }
    }

    pub fn sync_state (&mut self, mbc: &mut Mbc) {
        if self.is_pending_joypad_interrupt_trigger {
            //print!("executing mbc.hw_reg.set_if_joypad_bit4()\n");
            mbc.hw_reg.set_if_joypad_bit4();
            self.is_pending_joypad_interrupt_trigger = false;
        }

        if mbc.is_joypad_pending_update_from_reg {
            //println!("updating joypad from reg");

            let byte = mbc.hw_reg.joyp;

            if byte & 0x20 == 0x20 {
                self.select_buttons = false;
                self.select_dpad = true;
            }
            else if byte & 0x10 == 0x10 {
                self.select_dpad = false;
                self.select_buttons = true;
            }
            let new_byte = self.get_state_as_u8();

            mbc.hw_reg.joyp = new_byte;

            mbc.is_joypad_pending_update_from_reg = false;
        }

        // else if self.is_reg_pending_update_from_joypad {
        //     //println!("updating joypad reg from obj");
        //     let lower_joyp = self.get_state_as_u8();
        //     let upper_joyp =  mbc.hw_reg.joyp & 0b0011_0000;
        //     mbc.hw_reg.joyp = upper_joyp | lower_joyp;
        //     self.is_reg_pending_update_from_joypad = false;
        // }


    }


    pub fn handle_input(&mut self, key: KeyCode, state: ElementState) {
        // interrupt only triggered when bit goes from 1 to 0 (key pressed)
        if state == ElementState::Pressed {
            //println!("ElementState is Pressed, setting is_pending_joypad_interrupt_trigger to true");

            match key {
                KeyCode::KeyW => {
                    if !self.up {
                        self.up = true;
                        println!("pressed up");
                    }
                },
                KeyCode::KeyA => {
                    if !self.left {
                        self.left = true;
                        println!("pressed left");
                    }
                },
                KeyCode::KeyS => {
                    if !self.down {
                        self.down = true;
                        println!("pressed down");
                    }
                },
                KeyCode::KeyD => {
                    if !self.right {
                        self.right = true;
                        println!("pressed right");
                    }
                },
                KeyCode::KeyK => {
                    if !self.b {
                        self.b = true;
                        println!("pressed b");
                    }
                },
                KeyCode::KeyL => {
                    if !self.a {
                        self.a = true;
                        println!("pressed a");
                    }
                },
                KeyCode::Backspace => {
                    if !self.select {
                        self.select = true;
                        println!("pressed select");
                    }
                },
                KeyCode::Enter => {
                    if !self.start {
                        self.start = true;
                        println!("pressed start");
                    }
                },
                _ => {
                    println!("unrecognized key in Joypad.handle_input()");
                }
            }
            self.is_pending_joypad_interrupt_trigger = true;
        }
        else { // release key
            match key {
                KeyCode::KeyW => {
                    self.up = false;
                    println!("released up");
                },
                KeyCode::KeyA => {
                    self.left = false;
                    println!("released left");
                },
                KeyCode::KeyS => {
                    self.down = false;
                    println!("released down");
                },
                KeyCode::KeyD => {
                    self.right = false;
                    println!("released right");
                },
                KeyCode::KeyK => {
                    self.b = false;
                    println!("released b");
                },
                KeyCode::KeyL => {
                    self.a = false;
                    println!("released a");
                },
                KeyCode::Backspace => {
                    self.select = false;
                    println!("released select");
                },
                KeyCode::Enter => {
                    self.start = false;
                    println!("released start");
                },
                _ => {
                    println!("unrecognized key in Joypad.handle_input()");
                }
            }
        }
        self.is_reg_pending_update_from_joypad = true;
    }

    pub fn get_state_as_u8(&self) -> u8 {
        let mut state: u8 = 0x3F;

        if self.select_dpad {
            state &= 0b0010_1111;
            if self.right == true {
                state &= 0b0010_1110;
            }
            if self.left == true {
                state &= 0b0010_1101;
            }
            if self.up == true {
                state &= 0b0010_1011;
            }
            if self.down == true {
                state &= 0b0010_0111
            }
        }
        else if self.select_buttons {
            state &= 0b0001_1111;
                if self.a == true {
                    state &= 0b0001_1110;
                }
                if self.b == true {
                    state &= 0b0001_1101;
                }
                if self.select == true {
                    state &= 0b0001_1011;
                }
                if self.start == true {
                    state &= 0b0001_0111
                }
        }

        state
    }
}