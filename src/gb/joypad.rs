

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

    pub is_reg_pending_update_from_obj: bool,
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
            is_reg_pending_update_from_obj: false,
            is_pending_joypad_interrupt_trigger: false,
        }
    }

    pub fn sync_state (&mut self, mbc: &mut Mbc) {
        if self.is_pending_joypad_interrupt_trigger {
            //print!("executing mbc.hw_reg.set_if_joypad_bit4()\n");
            mbc.hw_reg.set_if_joypad_bit4();
            self.is_pending_joypad_interrupt_trigger = false;
        }

        if self.is_reg_pending_update_from_obj {
            //println!("updating joypad reg from obj");
            let lower_joyp = self.get_state_as_u8();
            let upper_joyp =  mbc.hw_reg.joyp & 0b1111_0000;
            mbc.hw_reg.joyp = upper_joyp | lower_joyp;
            self.is_reg_pending_update_from_obj = false;
        }

        else if mbc.is_joypad_pending_update_from_reg {
           //println!("updating joypad from reg");

            let byte = mbc.hw_reg.joyp;
            // if byte == 0xFF {
            //     self.select_buttons = false;
            //     self.select_dpad = false;
            //     self.a = false;
            //     self.b = false;
            //     self.select = false;
            //     self.start = false;
            //     self.right = false;
            //     self.left = false;
            //     self.up = false;
            //     self.down = false;
            // }
            if byte & 0x20 == 0x00 {
                self.select_buttons = true;
                self.select_dpad = false;
            }
            else if byte & 0x10 == 0x00 {
                self.select_dpad = true;
                self.select_buttons = false;
            }


            mbc.is_joypad_pending_update_from_reg = false;
        }
    }

    // todo handle key press VS release
    pub fn handle_input(&mut self, key: KeyCode, state: ElementState) {
        // interrupt only triggered when bit goes from 1 to 0 (key pressed)
        if state == ElementState::Pressed {
            //println!("ElementState is Pressed, setting is_pending_joypad_interrupt_trigger to true");

            match key {
                KeyCode::KeyW => {
                    if !self.up {
                        self.up = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed up");
                    }

                },
                KeyCode::KeyA => {
                    if !self.left {
                        self.left = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed left");
                    }
                },
                KeyCode::KeyS => {
                    if !self.down {
                        self.down = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed down");
                    }
                },
                KeyCode::KeyD => {
                    if !self.right {
                        self.right = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed right");

                    }
                },
                KeyCode::KeyK => {
                    if !self.b {
                        self.b = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed b");
                    }
                },
                KeyCode::KeyL => {
                    if !self.a {
                        self.a = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed a");
                    }
                },
                KeyCode::Backspace => {
                    if !self.select {
                        self.select = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed select");
                    }
                },
                KeyCode::Enter => {
                    if !self.start {
                        self.start = true;
                        self.is_pending_joypad_interrupt_trigger = true;
                        println!("pressed start");
                    }
                },
                _ => {
                    println!("unrecognized key in Joypad.handle_input()");
                }
            }
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

        self.is_reg_pending_update_from_obj = true;

    }

    pub fn get_state_as_u8(&self) -> u8 {
        let mut state: u8 = 0xFF;

        if self.select_dpad {
            state &= 0b1110_1111;
            if self.right == true {
                state &= 0b1111_1110;
            }
            if self.left == true {
                state &= 0b1111_1101;
            }
            if self.up == true {
                state &= 0b1111_1011;
            }
            if self.down == true {
                state &= 0b1111_0111
            }
        }
        else if self.select_buttons {
            state &= 0b1101_1111;
                if self.a == true {
                    state &= 0b1111_1110;
                }
                if self.b == true {
                    state &= 0b1111_1101;
                }
                if self.select == true {
                    state &= 0b1111_1011;
                }
                if self.start == true {
                    state &= 0b1111_0111
                }
        }

        state
    }
}