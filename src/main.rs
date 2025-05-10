
mod gb;

use gb::bios::ColorMode;

use crate::gb::emu::*;
use crate::gb::bios::*;

fn main() {
    let mut emu = Emu::new(ColorMode::Gray);
    emu.load_rom(String::from("pokemon_red_gbc.gb"));
    emu.run();
}
