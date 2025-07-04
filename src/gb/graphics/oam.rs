
#[derive(Default)]
pub struct Oam {
    pub byte0_y_pos: u8,
    pub byte1_x_pos: u8,
    pub byte2_tile_num: u8,
    pub byte3_sprite_flags: u8,
}

impl Oam {
    pub fn get_byte3_bit_4(&self) -> bool {
        if self.byte3_sprite_flags & 0b0001_0000 == 0b0001_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_4(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0001_0000;
    }

    pub fn get_byte3_bit_5(&self) -> bool {
        if self.byte3_sprite_flags & 0b0010_0000 == 0b0010_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_5(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0010_0000;
    }

    pub fn get_byte3_bit_6(&self) -> bool {
        if self.byte3_sprite_flags & 0b0100_0000 == 0b0100_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_6(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0100_0000;
    }

    pub fn get_byte3_bit_7(&self) -> bool {
        if self.byte3_sprite_flags & 0b1000_0000 == 0b1000_0000 {
            true
        }
        else {
            false
        }
    }
    pub fn set_byte3_bit_7(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b1000_0000;
    }
}

