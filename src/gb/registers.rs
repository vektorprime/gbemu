



pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
    pub fn get_a(&self) -> u8 {
        self.a
    }

    pub fn set_a(&mut self, val: u8) {
        self.a = val;
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }

    pub fn set_b(&mut self, val: u8) {
        self.b = val;
    }

    pub fn get_c(&self) -> u8 {
        self.c
    }

    pub fn set_c(&mut self, val: u8) {
        self.c = val;
    }

    pub fn get_d(&self) -> u8 {
        self.d
    }

    pub fn set_d(&mut self, val: u8) {
        self.d = val;
    }

    pub fn get_e(&self) -> u8 {
        self.e
    }

    pub fn set_e(&mut self, val: u8) {
        self.e = val;
    }

    pub fn get_f(&self) -> u8 {
        self.f
    }

    pub fn set_f(&mut self, val: u8) {
        self.f = val;
    }

    pub fn get_h(&self) -> u8 {
        self.h
    }

    pub fn set_h(&mut self, val: u8) {
        self.h = val;
    }

    pub fn get_l(&self) -> u8 {
        self.l
    }

    pub fn set_l(&mut self, val: u8) {
        self.l = val;
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, val: u16) {
        self.sp = val;
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (self.f as u16)
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = ((val & 0xFF00) >> 8) as u8;
        self.f = (val & 0xFF) as u8;
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = ((val & 0xFF00) >> 8) as u8;
        self.c = (val & 0xFF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = ((val & 0xFF00) >> 8) as u8;
        self.e = (val & 0xFF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = ((val & 0xFF00) >> 8) as u8;
        self.l = (val & 0xFF) as u8;
    }

    pub fn get_and_inc_pc(&mut self) -> u16 {
        let ret_pc = self.pc;
        self.pc += 1;
        ret_pc
    }

    pub fn inc_pc(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }
}