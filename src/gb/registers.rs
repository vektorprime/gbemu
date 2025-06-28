


pub const INVERSE_Z_FLAG_BITS: u8 = 0b0111_1111;
pub const INVERSE_N_FLAG_BITS: u8 = 0b1011_1111;
pub const INVERSE_H_FLAG_BITS: u8 = 0b1101_1111;
pub const INVERSE_C_FLAG_BITS: u8 = 0b1110_1111;

pub const Z_FLAG_BITS: u8 = 0b1000_0000;
pub const N_FLAG_BITS: u8 = 0b0100_0000;
pub const H_FLAG_BITS: u8 = 0b0010_0000;
pub const C_FLAG_BITS: u8 = 0b0001_0000;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FlagBits {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InverseFlagBits {
    Z = 0b0111_1111,
    N = 0b1011_1111,
    H = 0b1101_1111,
    C = 0b1110_1111,
}

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

    pub fn handle_flags(&mut self, inst_name: &str) {
        if inst_name.contains("ADD") || inst_name.contains("INC") || inst_name.contains("ADC") || inst_name.contains("OR ") || inst_name.contains("AND") {
            self.clear_n_flag();
        }
        else if inst_name.contains("SUB") || inst_name.contains("DEC") || inst_name.contains("SBC") || inst_name.contains("CP") {
            self.set_n_flag();
        }
        else if inst_name.contains("RLCA") || inst_name.contains("RLA") || inst_name.contains("RRCA") || inst_name.contains("RRA") {
            self.clear_z_flag();
            self.clear_n_flag();
            self.clear_h_flag();
            // handle C in inst code
        }
        else if inst_name.contains("SWAP") {
            self.clear_c_flag();
            self.clear_n_flag();
            self.clear_h_flag();
            // handle Z in inst code
        }
        else if inst_name.contains("RLC") {
            self.clear_n_flag();
            self.clear_h_flag();
            // handle Z in inst code
        }
        else if inst_name.contains("BIT") {
            self.clear_n_flag();
            self.set_h_flag();
            // handle Z in inst code
        }
    }

    pub fn is_z_flag_set(&self) -> bool {
        let mut current_flags = self.get_f();
        current_flags &= Z_FLAG_BITS;
        if current_flags == Z_FLAG_BITS {
            true
        }
        else {
            false
        }
    }

    pub fn set_z_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags |= Z_FLAG_BITS;
        self.set_f(current_flags);
    }
    
    pub fn clear_z_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags &= INVERSE_Z_FLAG_BITS;
        self.set_f(current_flags);
    }

    pub fn clear_n_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags &= INVERSE_N_FLAG_BITS;
        self.set_f(current_flags);
    }



    pub fn clear_h_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags &= INVERSE_H_FLAG_BITS;
        self.set_f(current_flags);
    }

    pub fn set_h_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags |= H_FLAG_BITS;
        self.set_f(current_flags);
    }

    pub fn is_h_flag_set(&self) -> bool {
        let mut current_flags = self.get_f();
        current_flags &= H_FLAG_BITS;
        if current_flags == H_FLAG_BITS {
            true
        }
        else {
            false
        }
    }

    pub fn get_c_flag(&mut self)  -> u8 {
        let mut current_flags = self.get_f();
        current_flags &= C_FLAG_BITS;
        current_flags
    }
 
    pub fn is_c_flag_set(&self) -> bool {
        let mut current_flags = self.get_f();
        current_flags &= C_FLAG_BITS;
        if current_flags == C_FLAG_BITS {
            true
        }
        else {
            false
        }
    }
    
    pub fn clear_c_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags &= INVERSE_C_FLAG_BITS;
        self.set_f(current_flags);
    }
    pub fn set_c_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags |= C_FLAG_BITS;
        self.set_f(current_flags);
    }

    pub fn set_n_flag(&mut self)  {
        let mut current_flags = self.get_f();
        current_flags |= N_FLAG_BITS;
        self.set_f(current_flags);
    }

    pub fn is_n_flag_set(&self) -> bool {
        let mut current_flags = self.get_f();
        current_flags &= N_FLAG_BITS;
        if current_flags == N_FLAG_BITS {
            true
        }
        else {
            false
        }
    }

    pub fn get_a(&self) -> u8 {
        self.a
    }

    pub fn set_a(&mut self, val: u8) {
        self.a = val;
    }

    pub fn inc_a(&mut self) {
        // wrap on overflow 
        self.a = self.a.wrapping_add(1);
        if self.a == 0 {
            self.set_z_flag()
        }
    }

    pub fn dec_a(&mut self) {
       // wrap on underflow
        self.a = self.a.wrapping_sub(1);
        if self.a == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }

    pub fn set_b(&mut self, val: u8) {
        self.b = val;
    }

    pub fn inc_b(&mut self) {
        // wrap on overflow 
        self.b = self.b.wrapping_add(1);
        if self.b == 0 {
            self.set_z_flag()
        }
    }

    pub fn dec_b(&mut self) {
        // wrap on underflow
        self.b = self.b.wrapping_sub(1);
        if self.b == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
    }

    pub fn get_c(&self) -> u8 {
        self.c
    }

    pub fn set_c(&mut self, val: u8) {
        self.c = val;
    }

    pub fn inc_c(&mut self) {
        // wrap on overflow 
        self.c = self.c.wrapping_add(1);
        if self.c == 0 {
            self.set_z_flag()
        }
    }

    pub fn dec_c(&mut self) {
        // wrap on underflow
        self.c = self.c.wrapping_sub(1);
        if self.c == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
    }

    pub fn get_d(&self) -> u8 {
        self.d
    }

    pub fn set_d(&mut self, val: u8) {
        self.d = val;
    }

    
    pub fn inc_d(&mut self) {
        // wrap on overflow 
        self.d = self.d.wrapping_add(1);
        if self.d == 0 {
            self.set_z_flag()
        }
    }

    pub fn dec_d(&mut self) {
        // wrap on underflow
        self.d = self.d.wrapping_sub(1);
        //print!("reg d is {}\n", self.d);
        if self.d == 0 {
            //print!("setting z flag");
            self.set_z_flag()
        }
        else {
            //print!("clearing z flag\n");
            self.clear_z_flag();
        }
    }

    pub fn get_e(&self) -> u8 {
        self.e
    }

    pub fn set_e(&mut self, val: u8) {
        self.e = val;
    }

    pub fn inc_e(&mut self) {
        // wrap on overflow 
        self.e = self.e.wrapping_add(1);
        if self.e == 0 {
            self.set_z_flag()
        }
    }
    pub fn dec_e(&mut self) {
        // wrap on underflow
        self.e = self.e.wrapping_sub(1);
        if self.e == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
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

    pub fn inc_h(&mut self) {
        // wrap on overflow 
        self.h = self.h.wrapping_add(1);
        if self.h == 0 {
            self.set_z_flag()
        }
    }
 
    pub fn dec_h(&mut self) {
        // wrap on overflow 
        self.h = self.h.wrapping_sub(1);
        if self.h == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
    }

    pub fn get_l(&self) -> u8 {
        self.l
    }

    pub fn set_l(&mut self, val: u8) {
        self.l = val;
    }

    pub fn inc_l(&mut self) {
        // wrap on overflow 
        self.l = self.l.wrapping_add(1);
        if self.l == 0 {
            self.set_z_flag()
        }
    }

    pub fn dec_l(&mut self) {
       // wrap on underflow
        self.l = self.l.wrapping_sub(1);
        if self.l == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, val: u16) {
        self.sp = val;
    }

    pub fn inc_sp(&mut self) {
        // wrap on overflow 
        self.sp = self.sp.wrapping_add(1); 
    }
    pub fn dec_sp(&mut self) {
        // wrap on overflow 
        self.sp = self.sp.wrapping_sub(1);
        if self.sp == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
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

    pub fn set_af_with_two_val(&mut self, val1: u8, val2: u8) {
        self.a = val2;
        self.f = val1;
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn inc_bc(&mut self) {
        let mut current = (self.b as u16) << 8 | (self.c as u16);
        //inc should not handle overflows
        current = current.wrapping_add(1);
        self.set_bc(current);

    }

    pub fn inc_de(&mut self) {
        let mut current = (self.d as u16) << 8 | (self.e as u16);
        //inc should not handle overflows
        current = current.wrapping_add(1);
        self.set_de(current);

    }

    pub fn dec_de(&mut self) {
        let mut current = (self.d as u16) << 8 | (self.e as u16);
        //dec should not handle underflows
        current = current.wrapping_sub(1);
        if self.get_de() == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
        self.set_de(current);
    }

    pub fn dec_bc(&mut self) {
        let mut current = (self.b as u16) << 8 | (self.c as u16);
        //dec should not handle underflows
        current = current.wrapping_sub(1);
        if self.get_bc() == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
        self.set_bc(current);
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = ((val & 0xFF00) >> 8) as u8;
        self.c = (val & 0xFF) as u8;
    }

    pub fn set_bc_with_two_val(&mut self, val1: u8, val2: u8) {
        self.b = val2;
        self.c = val1;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = ((val & 0xFF00) >> 8) as u8;
        self.e = (val & 0xFF) as u8;
    }

    pub fn set_de_with_two_val(&mut self, val1: u8, val2: u8) {
        self.d = val2;
        self.e = val1;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = ((val & 0xFF00) >> 8) as u8;
        self.l = (val & 0xFF) as u8;
    }


    pub fn inc_hl(&mut self) {
        let mut current = (self.h as u16) << 8 | (self.l as u16);
        //inc should not handle overflows
        current = current.wrapping_add(1);
        self.set_hl(current);
    }

    pub fn set_hl_with_two_val(&mut self, val1: u8, val2: u8) {
        self.h = val2;
        self.l = val1;
    }


    pub fn dec_hl(&mut self) {
        let mut current = (self.h as u16) << 8 | (self.l as u16);
        //dec should not handle underflows
        current = current.wrapping_sub(1);
        if self.get_hl() == 0 {
            self.set_z_flag()
        }
        else {
            self.clear_z_flag();
        }
        self.set_hl(current);
    }

    pub fn get_and_inc_pc(&mut self) -> u16 {
        let ret_pc = self.pc;
        self.pc = self.pc.wrapping_add(1);
        ret_pc
    }

    pub fn inc_pc_by_val(&mut self, size: u16) {
        self.pc += size;
    }

    pub fn inc_pc_by_inst_val(&mut self, mut size: u8) {
        //pc was already incremented once during get_next_op
        if size > 1 {
            size -= 1;
            self.pc += size as u16;
        }
    }

    pub fn inc_pc(&mut self) -> u16 {
        self.pc = self.pc.wrapping_add(1);
        self.pc
    }

    pub fn add_8bit(&mut self, a: u8, b: u8) -> u8 {
        // check for 8 bit overflow and set c flag
        let (result, overflowed) = a.overflowing_add(b);
        if overflowed {
            self.set_c_flag();
        }
        else {
            self.clear_c_flag();
        }
        if result == 0 {
            self.set_z_flag();
        }
        else {
            self.clear_z_flag();
        }

        // check for half-carry
        let half_a = a & 0b0000_1111;
        let half_b = b & 0b0000_1111;
        if half_a + half_b > 0b0000_1111 {
            self.set_h_flag();
        }
        else {
            self.clear_h_flag();
        }
        
        // always set val whether overflow or not
        result
    }

    pub fn add_16bit(&mut self, a: u16, b: u16) -> u16 {
        // check for 16 bit overflow and set c flag
        let (result, overflowed) = a.overflowing_add(b);
        if overflowed {
            self.set_c_flag();
        }
        else {
            self.clear_c_flag();
        }
        if result == 0 {
            self.set_z_flag();
        }
        else {
            self.clear_z_flag();
        }

        // check for half-carry
        let half_a = a & 0b0000_1111_1111_1111;
        let half_b = b & 0b0000_1111_1111_1111;
        if half_a + half_b > 0b0000_1111_1111_1111 {
            self.set_h_flag();
        }
        else {
            self.clear_h_flag();
        }

        // always return the overflow val
        result
    }

    pub fn sub_8bit(&mut self, a: u8, b: u8) -> u8 {
        // check for 8 bit underflow and set c flag
        let (result, underflowed) = a.overflowing_sub(b);
        if underflowed {
            self.set_c_flag();
        }
        else {
            self.clear_c_flag();
        }

        if result == 0 {
            self.set_z_flag();
        }
        else {
            self.clear_z_flag();
        }

        // // check for half-carry attempt
        let half_a = a & 0b0000_1111;
        let half_b = b & 0b0000_1111;
        if half_a < half_b {
            self.set_h_flag();
        }
        else {
            self.clear_h_flag();
        }

        // always set val whether overflow or not
        result
    }

    pub fn sub_8bit_carry(&mut self, a: u8, b: u8) -> u8 {
        // check for 8 bit underflow and set c flag
        let c_bool = self.is_c_flag_set();
        let c = if c_bool { 1 } else { 0 };
        let (mut result, underflowed) = a.overflowing_sub(b);
        result = result.wrapping_sub(c);
        if underflowed {
            self.set_c_flag();
        }
        else {
            self.clear_c_flag();
        }

        if result == 0 {
            self.set_z_flag();
        }
        else {
            self.clear_z_flag();
        }

        // // check for half-carry attempt
        let half_a = a & 0b0000_1111;
        let half_b = b & 0b0000_1111;
        if half_a < half_b {
            self.set_h_flag();
        }
        else {
            self.clear_h_flag();
        }

        // always set val whether overflow or not
        result
    }

    pub fn sub_16bit(&mut self, a: u16, b: u16) -> u16 {
        // check for 16 bit overflow and set c flag
        let (result, underflowed) = a.overflowing_sub(b);
        if underflowed {
            self.set_h_flag();
        }
        else {
            self.clear_h_flag();
        }
        if result == 0 {
            self.set_z_flag();
        }
        else {
            self.clear_z_flag();
        }


        // check for 12 bit half-carry
        let half_a = a & 0b0000_1111_1111_1111;
        let half_b = b & 0b0000_1111_1111_1111;
        if half_a < half_b {
            self.set_h_flag();
        }
        else {
            self.clear_h_flag();
        }
        // always return the overflow val
        result
    }

}