

pub struct Joypad {
    pub a_right: bool,
    pub b_left: bool,
    pub select_up: bool,
    pub start_down: bool,
    pub select_dpad: bool,
    pub select_buttons: bool,
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
        }
    }

    pub fn get_joypad_state(&self) -> u8 {
        let mut state: u8 = 0xFF;
        if self.select_dpad == true {
            state &= 0b1110_1111;
        }
        else if self.select_buttons == true {
            state &= 0b1101_1111;
        }

        if self.a_right == true {
            state &= 0b1111_1110;
        }
        if self.b_left == true {
            state &= 0b1111_1101;
        }
        if self.select_up == true {
            state &= 0b1111_1011;
        }
        state
    }
}