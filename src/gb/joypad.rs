

use winit::keyboard::KeyCode;
use crate::gb::mbc::Mbc;

use winit::event::ElementState;

pub struct Joypad {
    pub a_right: bool,
    pub b_left: bool,
    pub select_up: bool,
    pub start_down: bool,
    pub select_dpad: bool,
    pub select_buttons: bool,
    pub is_reg_pending_update_from_obj: bool,
    pub is_pending_joypad_interrupt_trigger: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            a_right: false,
            b_left: false,
            select_up: false,
            start_down: false,
            select_dpad: false,
            select_buttons: false,
            is_reg_pending_update_from_obj: false,
            is_pending_joypad_interrupt_trigger: false,
        }
    }

    pub fn sync_state (&mut self, mbc: &mut Mbc) {
        if self.is_pending_joypad_interrupt_trigger {
            print!("executing mbc.hw_reg.set_if_joypad_bit4()\n");
            mbc.hw_reg.set_if_joypad_bit4();
            //mbc.hw_reg.set_ie_joypad_bit4();
            self.is_pending_joypad_interrupt_trigger = false;
        }

        if self.is_reg_pending_update_from_obj {
            println!("updating joypad reg from obj");
            let lower_joyp = self.get_state_as_u8();
            let upper_joyp =  mbc.hw_reg.joyp & 0b0011_0000;
            mbc.hw_reg.joyp = upper_joyp | lower_joyp;
            self.is_reg_pending_update_from_obj = false;
        }

        else if mbc.is_joypad_pending_update_from_reg {
            println!("updating joypad from reg");

            let byte = mbc.hw_reg.joyp;
            if byte & 0x30 == 0x30 {
                self.select_buttons = false;
                self.select_dpad = false;
                self.a_right = false;
                self.b_left = false;
                self.select_up = false;
                self.start_down = false;
            }
            else if byte & 0x20 == 0x00 {
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
            println!("ElementState is Pressed, setting is_pending_joypad_interrupt_trigger to true");
            self.is_pending_joypad_interrupt_trigger = true;

            match key {
                KeyCode::KeyW => {
                    self.select_up = true;
                    println!("pressed select or up");
                },
                KeyCode::KeyA => {
                    self.b_left = true;
                    println!("pressed b or left");
                },
                KeyCode::KeyS => {
                    self.start_down = true;
                    println!("pressed start or down");
                },
                KeyCode::KeyD => {
                    self.a_right = true;
                    println!("pressed a or right");
                },
                KeyCode::Backspace => {
                    self.select_up = true;
                    println!("pressed select or up");
                },
                KeyCode::Enter => {
                    self.start_down = true;
                    println!("pressed start or down");
                },
                _ => {
                    println!("unrecognized key in Joypad.handle_input()");
                }
            }

            // just for debugging
            if self.select_buttons {
                if self.select_up == true {
                    println!("pressed up on dpad");
                }
                if self.b_left == true {
                    println!("pressed left on dpad");
                }
                if self.start_down == true {
                    println!("pressed down on dpad");
                }
                if self.a_right == true {
                    println!("pressed right on dpad");
                }

            }
            else if self.select_dpad {
                if self.select_up == true {
                    println!("pressed select button");
                }
                if self.b_left == true {
                    println!("pressed b button");
                }
                if self.start_down == true {
                    println!("pressed start button");
                }
                if self.a_right == true {
                    println!("pressed a button");
                }
            }
        }
        else { // release key
            match key {
                KeyCode::KeyW => {
                    self.select_up = false;
                    println!("released select or up");
                },
                KeyCode::KeyA => {
                    self.b_left = false;
                    println!("released b or left");
                },
                KeyCode::KeyS => {
                    self.start_down = false;
                    println!("released start or down");

                },
                KeyCode::KeyD => {
                    self.a_right = false;
                    println!("released a or right");
                },
                KeyCode::Backspace => {
                    self.select_up = false;
                    println!("released select or up");
                },
                KeyCode::Enter => {
                    self.start_down = false;
                    println!("released start or down");
                },
                _ => {
                    println!("unrecognized released key in Joypad.handle_input()");
                }
            }
        }

        self.is_reg_pending_update_from_obj = true;

    }

    pub fn get_state_as_u8(&self) -> u8 {
        let mut state: u8 = 0xFF;
        if self.select_dpad || self.select_buttons {
            // if self.select_dpad == true {
            //     state &= 0b1110_1111;
            // }
            // else if self.select_buttons == true {
            //     state &= 0b1101_1111;
            // }

            if self.a_right == true {
                state &= 0b1111_1110;
            }
            if self.b_left == true {
                state &= 0b1111_1101;
            }
            if self.select_up == true {
                state &= 0b1111_1011;
            }
            if self.start_down == true {
                state &= 0b1111_0111
            }
        }

        state
    }
}