#[derive(Debug, Default, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Sprite {
    pub byte0_y_pos: u8,
    pub byte1_x_pos: u8,
    pub byte2_tile_num: u8,
    pub byte3_sprite_flags: u8,
}



impl Sprite {

    pub fn new() -> Self {
        Sprite {
            ..Default::default()
        }
    }

    pub fn get_byte3_sprite_flags_bit4_dmg_palette(&self) -> bool {
        if self.byte3_sprite_flags & 0b0001_0000 == 0b0001_0000 {
            true
        } else {
            false
        }
    }
    pub fn set_byte3_sprite_flags_bit4_dmg_palette(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0001_0000;
    }

    pub fn get_byte3_sprite_flags_bit5_xflip(&self) -> bool {
        if self.byte3_sprite_flags & 0b0010_0000 == 0b0010_0000 {
            true
        } else {
            false
        }
    }
    pub fn set_byte3_sprite_flags_bit5_xflip(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0010_0000;
    }

    pub fn get_byte3_sprite_flags_bit6_yflip(&self) -> bool {
        if self.byte3_sprite_flags & 0b0100_0000 == 0b0100_0000 {
            true
        } else {
            false
        }
    }
    pub fn set_byte3_sprite_flags_bit6_yflip(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b0100_0000;
    }

    pub fn get_byte3_sprite_flags_bit7_priority(&self) -> bool {
        if self.byte3_sprite_flags & 0b1000_0000 == 0b1000_0000 {
            true
        } else {
            false
        }
    }
    pub fn set_byte3_sprite_flags_bit7_priority(&mut self) {
        self.byte3_sprite_flags = self.byte3_sprite_flags | 0b1000_0000;
    }
}
