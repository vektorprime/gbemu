



pub struct HardwareRegisters {
    // LCD and scrolling
    pub lcdc: u8,  // FF40
    pub stat: u8,  // FF41
    pub scy: u8,   // FF42
    pub scx: u8,   // FF43
    pub ly: u8,    // FF44
    pub lyc: u8,   // FF45
    pub dma: u8,   // FF46
    pub bgp: u8,   // FF47
    pub obp0: u8,  // FF48
    pub obp1: u8,  // FF49
    pub wy: u8,    // FF4A
    pub wx: u8,    // FF4B

    // Boot ROM and interrupts
    pub boot_rom_control: u8, // FF50
    pub ie: u8,               // FFFF

    // Joypad and serial
    pub joyp: u8, // FF00
    pub sb: u8,   // FF01
    pub sc: u8,   // FF02

    // Timer
    pub div: u8,  // FF04
    pub tima: u8, // FF05
    pub tma: u8,  // FF06
    pub tac: u8,  // FF07

    // Interrupt flags
    pub if_reg: u8, // FF0F

    // Audio (NR10–NR52)
    pub nr10: u8, // FF10
    pub nr11: u8, // FF11
    pub nr12: u8, // FF12
    pub nr13: u8, // FF13
    pub nr14: u8, // FF14
    pub nr21: u8, // FF16
    pub nr22: u8, // FF17
    pub nr23: u8, // FF18
    pub nr24: u8, // FF19
    pub nr30: u8, // FF1A
    pub nr31: u8, // FF1B
    pub nr32: u8, // FF1C
    pub nr33: u8, // FF1D
    pub nr34: u8, // FF1E
    pub nr41: u8, // FF20
    pub nr42: u8, // FF21
    pub nr43: u8, // FF22
    pub nr44: u8, // FF23
    pub nr50: u8, // FF24
    pub nr51: u8, // FF25
    pub nr52: u8, // FF26

    // Wave pattern RAM
    pub wave_pattern: [u8; 16], // FF30–FF3F
}

impl HardwareRegisters {
    pub fn new() -> Self {
        HardwareRegisters {
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,

            boot_rom_control: 0,
            ie: 0,

            joyp: 0,
            sb: 0,
            sc: 0,

            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,

            if_reg: 0,

            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,

            wave_pattern: [0; 16],
        }
    }


}