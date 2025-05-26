



pub struct HardwareRegisters {
    lcdc: u8,
    bgp: u8,
    ly: u8,
}

impl HardwareRegisters {
    pub fn new() -> Self {
        HardwareRegisters {
            lcdc: 0, //FF40
            bgp: 0, //FF47
            ly:0, 
            
        }
    }


}