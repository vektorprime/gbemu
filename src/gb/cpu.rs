use crate::gb::registers::{Registers, InverseFlagBits, FlagBits, INVERSE_C_FLAG_BITS, INVERSE_H_FLAG_BITS, INVERSE_N_FLAG_BITS, INVERSE_Z_FLAG_BITS};
use crate::gb::instructions::Instruction;
use crate::gb::ram::Ram;
use crate::gb::bios::*;
use std::collections::HashMap;



pub struct Cpu {
    pub registers: Registers,
    pub ime: bool, // interrupt master
    pub opcode: u8, // opcode of current inst.
    pub cycles: u64, // total cycles count
    pub halted: bool,
}


impl Cpu {

    pub fn new() -> Self {
        Cpu {
            registers: Registers::new(),
            ime: false,
            opcode: 0,
            cycles: 0,
            halted: false, 
        }
    }

    pub fn inc_cycles_by_inst_val(&mut self, size: u8) {
        self.cycles += size as u64;
    }

    pub fn run(&mut self, mem: &mut Ram) {
        let valid_instructions    =  Cpu::setup_inst();
        let valid_cb_instructions =  Cpu::setup_cb_inst();
        loop {
            if !self.halted {
                let mut opcode = self.fetch_next_inst(mem);
                //if CB, read another byte, else decode and execute
                let mut is_cb_opcode = false;
                if opcode == 0xCB {
                    is_cb_opcode = true;
                    opcode = self.fetch_next_inst(mem);
                }
                let inst = if is_cb_opcode {
                    valid_cb_instructions.get(&opcode).unwrap()
                } else {
                    valid_instructions.get(&opcode).unwrap()
                };
                self.execute_inst(inst, mem, is_cb_opcode);
            }
        }
    }

    pub fn fetch_next_inst(&mut self, mem: &Ram) -> u8 {
        let pc_reg = self.registers.get_and_inc_pc();
        mem.read(pc_reg)
    }



    pub fn execute_inst(&mut self,  inst: &Instruction, mem: &mut Ram, is_cb_opcode: bool) {
        // todo
        if !is_cb_opcode {
            match inst.opcode {
                0x00 => {
                    // NOP
                    // pc is already inc
                },
                0x01 => {
                    // LD BC D16
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    self.registers.set_bc(u16::from_le_bytes([lo, hi]));
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x02 => {
                    // LD (BC) A
                    let a = self.registers.get_a();
                    let bc = self.registers.get_bc();
                    mem.write(bc, a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x03 => {
                    // INC BC
                    self.registers.inc_bc();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x04 => {
                    // INC B
                    self.registers.inc_b();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x05 => {
                    // DEC B
                    self.registers.dec_b();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x06 => {
                    // LD B D8
                    let operand = mem.read(self.registers.get_pc());
                    self.registers.set_b(operand);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x07 => {
                    // RLCA
                    let mut bit0 = 0b0000_0001;
                    let mut a_reg = self.registers.get_a();
                    bit0 &= a_reg;
                    a_reg <<= 1;
                    if bit0 == 1 {
                        a_reg |= bit0;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    self.registers.set_a(a_reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x08 => {
                    // LD (A16) SP
                    let sp = self.registers.get_sp();
                    
                    let lower_8bits = 0b0000_0000_1111_1111;
                    let lower_sp = sp & lower_8bits;
                    let dst_addr_p1 = mem.read(self.registers.get_pc());
                    let dst_addr_p2 = mem.read(self.registers.get_pc() + 1);
                    let dst_addr = u16::from_le_bytes([dst_addr_p1, dst_addr_p2]);
                    mem.write(dst_addr, lower_sp as u8);

                    let upper_sp = (sp >> 8) as u8;
                    mem.write(dst_addr + 1, upper_sp);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x09 => {
                    // ADD HL BC
                    let a = self.registers.get_hl();
                    let b = self.registers.get_bc();
                    let result = self.registers.add_16bit(a, b);
                    self.registers.set_hl(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0A => {
                    // LD A (BC)
                    let bc_deref = mem.read(self.registers.get_bc());
                    self.registers.set_a(bc_deref);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0B => {
                    // DEC BC
                    self.registers.dec_bc();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0C => {
                    // INC C
                    self.registers.inc_c();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0D => {
                    // DEC C
                    self.registers.dec_c();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0E => {
                    // LD C D8
                    let operand = mem.read(self.registers.get_pc());
                    self.registers.set_c(operand);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0F => {
                    // RRCA
                    let mut bit0 = 0b0000_0001;
                    let bit7 = 0b1000_0000;
                    let mut a_reg = self.registers.get_a();
                    bit0 &= a_reg;
                    a_reg >>= 1;
                    if bit0 == 1 {
                        a_reg |= bit7;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    self.registers.set_a(a_reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x10 => {
                    // STOP
                    // todo
                    // not sure how to handle this cleanly yet.
                    // maybe a loop that waits for button press
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x11 => {
                    // LD DE D16
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    self.registers.set_de(u16::from_le_bytes([lo, hi]));
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x12 => {
                    // LD (DE) A
                    let a = self.registers.get_a();
                    let de = self.registers.get_de();
                    mem.write(de, a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x13 => {
                    // INC DE
                    self.registers.inc_de();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x14 => {
                    // INC D
                    self.registers.inc_d();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x15 => {
                    // DEC D
                    self.registers.dec_d();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x16 => {
                    // LD D D8
                    let operand = mem.read(self.registers.get_pc());
                    self.registers.set_d(operand);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x17 => {
                    // RLA
                    let c_flag = self.registers.get_c_flag();
                    let mut a_reg = self.registers.get_a();
                    a_reg <<= 1;
                    if c_flag > 0 {
                        a_reg |= 0b0000_0001;
                    }
                    self.registers.set_a(a_reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x18 => {
                    // JR S8
                    let current_pc = self.registers.get_pc();
                    let new_pc = u16::from(mem.read(self.registers.get_pc() + 1));
                    self.registers.set_pc(current_pc + new_pc);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    // don't inc pc here because we jumped
                },
                0x19 => {
                    // ADD HL DE
                    let first_operand = self.registers.get_hl();
                    let second_operand = self.registers.get_de();

                    let (new_val, overflowed) = first_operand.overflowing_add(second_operand);
                    // check for 12 bit overflow and set h flag
                    let overflowed_12bit_max = 4096;
                    if new_val > overflowed_12bit_max {
                        self.registers.set_h_flag();
                    }
                    // check for 16 bit overflow and set c flag
                    if overflowed {
                        self.registers.set_c_flag();
                    }
                    // always set val whether overflow or not
                    self.registers.set_hl(new_val);

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1A => {
                    // LD A, (DE)
                    let addr = self.registers.get_de();
                    let value = mem.read(addr);
                    self.registers.set_a(value);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1B => {
                    // DEC DE
                    self.registers.dec_de();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },    
                0x1C => {
                    // INC E
                    self.registers.inc_e();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1D => {
                    // DEC E
                    self.registers.dec_e();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1E => {
                    // LD E, d8
                    let value = mem.read(self.registers.get_pc());
                    self.registers.set_e(value);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },   
                0x1F => {
                    // RRA
                    let mut bit0 = 0b0000_0001;
                    let c_flag_bit = self.registers.is_c_flag_set();
                    let bit7 = match c_flag_bit {
                        true =>  0b1000_0000,
                        false => 0b0000_0000,
                    };
                    let mut a_reg = self.registers.get_a();
                    bit0 &= a_reg;
                    a_reg >>= 1;
                    a_reg |= bit7;
                    if bit0 == 1 {
                        self.registers.set_c_flag(); 
                    }
                    else {
                        self.registers.clear_c_flag();
                    }

                    self.registers.set_a(a_reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },       
                0x20 => {
                    // JR NZ S8
                    let z_flag = self.registers.is_z_flag_set();
                    if !z_flag {
                        let pc = self.registers.get_pc();
                        let pc_offset = mem.read(pc) as u16;
                        let new_pc = pc + pc_offset;
                        self.registers.set_pc(new_pc);
                    }
                    else {
                        self.registers.inc_pc_by_inst_val(inst.size);
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },     
                0x21 => {
                    // LD HL D16
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    self.registers.set_hl(u16::from_le_bytes([lo, hi]));
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x22 => {
                    // LD (HL+) A
                    let a = self.registers.get_a();
                    let hl = self.registers.get_hl();
                    mem.write(hl, a);
                    self.registers.inc_hl();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x23 => {
                    // INC DE
                    self.registers.inc_de();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x24 => {
                    // INC H
                    self.registers.inc_h();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x25 => {
                    // DEC H
                    self.registers.dec_h();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x26 => {
                    // LD H D8
                    let val = mem.read(self.registers.get_pc());
                    self.registers.set_h(val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x27 => {
                    // DAA
                    // note: assumes a is a uint8_t and wraps from 0xff to 0
                    let n_flag = self.registers.is_n_flag_set();
                    let h_flag = self.registers.is_h_flag_set();
                    let c_flag = self.registers.is_c_flag_set();
                    let mut a_reg = self.registers.get_a();
                    if !n_flag {  // after an addition, adjust if (half-)carry occurred or if result is out of bounds
                        if c_flag || a_reg > 0x99 { 
                            a_reg += 0x60;  
                            self.registers.set_c_flag();                        
                        }
                        if h_flag || (a_reg & 0x0f) > 0x09 { 
                            a_reg += 0x6; 
                        }
                    } 
                    else {  // after a subtraction, only adjust if (half-)carry occurred
                        if c_flag { 
                            a_reg -= 0x60; 
                        }
                        if h_flag { 
                            a_reg -= 0x6; 
                        }
                    }

                    self.registers.set_a(a_reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x28 => {
                    // JR Z S8
                    let z_flag = self.registers.is_z_flag_set();
                    if z_flag {
                        let pc = self.registers.get_pc();
                        let pc_offset = mem.read(pc) as u16;
                        let new_pc = pc + pc_offset;
                        self.registers.set_pc(new_pc);
                    }
                    else {
                        self.registers.inc_pc_by_inst_val(inst.size);
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },   
                0x29 => {
                    // ADD HL HL
                    let first_operand = self.registers.get_hl();
                    let second_operand = self.registers.get_hl();
                    
                    let (new_val, overflowed) = first_operand.overflowing_add(second_operand);
                    // check for 12 bit overflow and set h flag
                    let overflowed_12bit_max = 4096;
                    if new_val > overflowed_12bit_max {
                        self.registers.set_h_flag();
                    }
                    // check for 16 bit overflow and set c flag
                    if overflowed {
                        self.registers.set_c_flag();
                    }
                    // always set val whether overflow or not
                    self.registers.set_hl(new_val);
                    
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2A => {
                    // LD A (HL+)
                    let addr = self.registers.get_hl();
                    let value = mem.read(addr);
                    self.registers.set_a(value);
                    self.registers.set_hl(addr.wrapping_add(1));
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2B => {
                    // DEC HL
                    self.registers.dec_hl();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2C => {
                    // INC L
                    self.registers.inc_l();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2D => {
                    // DEC L
                    self.registers.dec_l();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2E => {
                    // LD L, d8
                    let value = mem.read(self.registers.get_pc());
                    self.registers.set_l(value);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2F => { 
                    // CPL 
                    let reg = self.registers.get_a();
                    let mut val  =  !reg;
                    self.registers.set_a(val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x30 => {
                    // JR NC S8
                    let c_flag = self.registers.is_c_flag_set();
                    if c_flag {
                        let pc = self.registers.get_pc();
                        let pc_offset = mem.read(pc) as u16;
                        let new_pc = pc + pc_offset;
                        self.registers.set_pc(new_pc);
                    }
                    else {
                        self.registers.inc_pc_by_inst_val(inst.size);
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },   
                0x31 => {
                    // LD SP D16
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    self.registers.set_sp(u16::from_le_bytes([lo, hi]));
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x32 => {
                    // LD (HL-) A
                    let a = self.registers.get_a();
                    let add = self.registers.get_hl();
                    mem.write(add, a);
                    self.registers.dec_hl();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x33 => {
                    // INC SP
                    self.registers.inc_sp();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x34 => {
                    // INC (HL)
                    let addr = self.registers.get_hl();
                    let value = mem.read(addr);
                    let result = value.wrapping_add(1);
                    mem.write(addr, result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x35 => {
                    // DEC (HL)
                    let addr = self.registers.get_hl();
                    let value = mem.read(addr);
                    let result = value.wrapping_sub(1);
                    mem.write(addr, result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x36 => {
                    // LD (HL), d8
                    let value = mem.read(self.registers.get_pc());
                    let addr = self.registers.get_hl();
                    mem.write(addr, value);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x37 => {
                    // SCF
                    self.registers.set_c_flag();
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x38 => {
                    // JR C S8
                    let flag = self.registers.is_c_flag_set();
                    if flag {
                        let pc = self.registers.get_pc();
                        let pc_offset = mem.read(pc) as u16;
                        let new_pc = pc + pc_offset;
                        self.registers.set_pc(new_pc);
                    }
                    else {
                        self.registers.inc_pc_by_inst_val(inst.size);
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },   
                0x39 => {
                    // ADD HL SP
                    let first_operand = self.registers.get_hl();
                    let second_operand = self.registers.get_sp();
                    
                    let (new_val, overflowed) = first_operand.overflowing_add(second_operand);
                    // check for 12 bit overflow and set h flag
                    let overflowed_12bit_max = 4096;
                    if new_val > overflowed_12bit_max {
                        self.registers.set_h_flag();
                    }
                    // check for 16 bit overflow and set c flag
                    if overflowed {
                        self.registers.set_c_flag();
                    }
                    // always set val whether overflow or not
                    self.registers.set_hl(new_val);
                    
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3A => {
                    // LD A (HL-) 
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    self.registers.set_a(val);
                    self.registers.dec_hl(); 
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3B => {
                    // DEC SP
                    self.registers.dec_sp();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3C => {
                    // INC A
                    self.registers.inc_a();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3D => {
                    // DEC A
                    self.registers.dec_a();
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3E => {
                    // LD A D8
                    let value = mem.read(self.registers.get_pc());
                    self.registers.set_a(value);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3F => { 
                    // CCF
                    let current_flag = self.registers.is_c_flag_set();
                    if current_flag {
                        self.registers.clear_c_flag();
                    }
                    else {
                        self.registers.set_c_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x40 => {
                    // LD B B
                    let reg = self.registers.get_b();
                    self.registers.set_b(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x41 => {
                    // LD B C
                    let reg = self.registers.get_c();
                    self.registers.set_b(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x42 => {
                    // LD B D
                    let reg = self.registers.get_d();
                    self.registers.set_b(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x43 => {
                    // LD B E
                    let reg = self.registers.get_e();
                    self.registers.set_b(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x44 => {
                    // LD B H
                    let reg = self.registers.get_h();
                    self.registers.set_b(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x45 => {
                    // LD B L
                    let reg = self.registers.get_l();
                    self.registers.set_b(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x46 => {
                    // LD B (HL)
                    let reg = self.registers.get_hl();
                    self.registers.set_b(mem.read(reg));
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x47 => {
                    // LD B A
                    let reg = self.registers.get_a();
                    self.registers.set_b(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x48 => {
                    // LD C B
                    let reg = self.registers.get_b();
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x49 => {
                    // LD C C
                    let reg = self.registers.get_c();
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4A => {
                    // LD C D
                    let reg = self.registers.get_d();
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4B => {
                    // LD C E
                    let reg = self.registers.get_e();
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4C => {
                    // LD C H
                    let reg = self.registers.get_h();
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4D => {
                    // LD C L
                    let reg = self.registers.get_l();
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4E => {
                    // LD C (HL)
                    let addr = self.registers.get_hl();
                    let reg = mem.read(addr);
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4F => {
                    // LD C A
                    let reg = self.registers.get_a();
                    self.registers.set_c(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x50 => {
                    // LD D B
                    let reg = self.registers.get_b();
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x51 => {
                    // LD D C
                    let reg = self.registers.get_c();
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x52 => {
                    // LD D D
                    let reg = self.registers.get_d();
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x53 => {
                    // LD D E
                    let reg = self.registers.get_e();
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x54 => {
                    // LD D H
                    let reg = self.registers.get_h();
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x55 => {
                    // LD D L
                    let reg = self.registers.get_l();
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x56 => {
                    // LD D (HL)
                    let addr = self.registers.get_hl();
                    let reg = mem.read(addr);
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x57 => {
                    // LD D A
                    let reg = self.registers.get_a();
                    self.registers.set_d(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x58 => {
                    // LD E B
                    let reg = self.registers.get_b();
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x59 => {
                    // LD E C
                    let reg = self.registers.get_c();
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5A => {
                    // LD E D
                    let reg = self.registers.get_d();
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5B => {
                    // LD E E
                    let reg = self.registers.get_e();
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5C => {
                    // LD E H
                    let reg = self.registers.get_h();
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5D => {
                    // LD E L
                    let reg = self.registers.get_l();
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5E => {
                    // LD E (HL)
                    let addr = self.registers.get_hl();
                    let reg = mem.read(addr);
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5F => {
                    // LD E A
                    let reg = self.registers.get_a();
                    self.registers.set_e(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x60 => {
                    // LD H B
                    let reg = self.registers.get_b();
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x61 => {
                    // LD H C
                    let reg = self.registers.get_c();
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x62 => {
                    // LD H D
                    let reg = self.registers.get_d();
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x63 => {
                    // LD H E
                    let reg = self.registers.get_e();
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x64 => {
                    // LD H H
                    let reg = self.registers.get_h();
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x65 => {
                    // LD H L
                    let reg = self.registers.get_l();
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x66 => {
                    // LD H (HL)
                    let addr = self.registers.get_hl();
                    let reg = mem.read(addr);
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x67 => {
                    // LD H A
                    let reg = self.registers.get_a();
                    self.registers.set_h(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x68 => {
                    // LD L B
                    let reg = self.registers.get_b();
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x69 => {
                    // LD L C
                    let reg = self.registers.get_c();
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6A => {
                    // LD L D
                    let reg = self.registers.get_d();
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6B => {
                    // LD L E
                    let reg = self.registers.get_e();
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6C => {
                    // LD L H
                    let reg = self.registers.get_h();
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6D => {
                    // LD L L
                    let reg = self.registers.get_l();
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6E => {
                    // LD L (HL)
                    let addr = self.registers.get_hl();
                    let reg = mem.read(addr);
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6F => {
                    // LD L A
                    let reg = self.registers.get_a();
                    self.registers.set_l(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x70 => {
                    // LD (HL) B
                    let addr = self.registers.get_hl();
                    mem.write(addr, self.registers.get_b());
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x71 => {
                    // LD (HL) C
                    let addr = self.registers.get_hl();
                    mem.write(addr, self.registers.get_c());
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x72 => {
                    // LD (HL) D
                    let addr = self.registers.get_hl();
                    mem.write(addr, self.registers.get_d());
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x73 => {
                    // LD (HL) E
                    let addr = self.registers.get_hl();
                    mem.write(addr, self.registers.get_e());
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x74 => {
                    // LD (HL) H
                    let addr = self.registers.get_hl();
                    mem.write(addr, self.registers.get_h());
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x75 => {
                    // LD (HL) L
                    let addr = self.registers.get_hl();
                    mem.write(addr, self.registers.get_l());
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x76 => {
                    // HALT
                    self.halted = true;
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x77 => {
                    // LD (HL) A
                    let addr = self.registers.get_hl();
                    mem.write(addr, self.registers.get_a());
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x78 => {
                    // LD A B
                    let reg = self.registers.get_b();
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x79 => {
                    // LD A C
                    let reg = self.registers.get_c();
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7A => {
                    // LD A D
                    let reg = self.registers.get_d();
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7B => {
                    // LD A E
                    let reg = self.registers.get_e();
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7C => {
                    // LD A H
                    let reg = self.registers.get_h();
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7D => {
                    // LD A L
                    let reg = self.registers.get_l();
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7E => {
                    // LD A (HL)
                    let addr = self.registers.get_hl();
                    let reg = mem.read(addr);
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7F => {
                    // LD A A
                    let reg = self.registers.get_a();
                    self.registers.set_a(reg);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x80 => {
                    // ADD A B
                    let a = self.registers.get_a();
                    let b = self.registers.get_b();
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x81 => {
                    // ADD A C
                    let a = self.registers.get_a();
                    let b = self.registers.get_c();
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x82 => {
                    // ADD A D
                    let a = self.registers.get_a();
                    let b = self.registers.get_d();
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x83 => {
                    // ADD A E
                    let a = self.registers.get_a();
                    let b = self.registers.get_e();
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x84 => {
                    // ADD A H
                    let a = self.registers.get_a();
                    let b = self.registers.get_h();
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x85 => {
                    // ADD A L
                    let a = self.registers.get_a();
                    let b = self.registers.get_l();
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x86 => {
                    // ADD A (HL)
                    let a = self.registers.get_a(); 
                    let hl = self.registers.get_hl();
                    let b = mem.read(hl);
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x87 => {
                    // ADD A A
                    let a = self.registers.get_a();
                    let b = self.registers.get_a();
                    let result = self.registers.add_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                 0x90 => {
                    //SUB A B
                    let a = self.registers.get_a();
                    let b = self.registers.get_b();
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x91 => {
                    // SUB A C
                    let a = self.registers.get_a();
                    let b = self.registers.get_c();
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x92 => {
                    // SUB A D
                    let a = self.registers.get_a();
                    let b = self.registers.get_d();
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x93 => {
                    // SUB A E
                    let a = self.registers.get_a();
                    let b = self.registers.get_e();
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x94 => {
                    // SUB A H
                    let a = self.registers.get_a();
                    let b = self.registers.get_h();
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x95 => {
                    // SUB A L
                    let a = self.registers.get_a();
                    let b = self.registers.get_l();
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x96 => { 
                    // SUB A (HL)
                    let a = self.registers.get_a();
                    let addr = self.registers.get_hl();
                    let b = mem.read(addr);
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x97 => {
                    // SUB A A
                    let a = self.registers.get_a();
                    let b = self.registers.get_a();
                    let result = self.registers.sub_8bit(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC3 => {
                    // JP A16
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    self.registers.set_pc(u16::from_le_bytes([lo, hi]));
                    self.inc_cycles_by_inst_val(inst.cycles);
                },

                _ => {
                    //todo
                }
            } // match
        }
        else { // cb opcodes
            match inst.opcode {
                0xC3 => {
                    // JP A16
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    self.registers.set_pc(u16::from_le_bytes([lo, hi]));
                    self.inc_cycles_by_inst_val(inst.cycles);
                },

                _ => {
                    //todo
                }
            }
        }
        
    } // fn

   
    pub fn setup_inst() -> HashMap<u8, Instruction> {
        // https://meganesu.github.io/generate-gb-opcodes/
        let mut all_instructions = HashMap::new();
        all_instructions.insert(0x00, Instruction {
            opcode: 0x00,
            name: "NOP",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x01, Instruction {
            opcode: 0x01,
            name: "LD BC D16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0x02, Instruction {
            opcode: 0x02,
            name: "LD (BC) A",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x03, Instruction {
            opcode: 0x03,
            name: "INC BC",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x04, Instruction {
            opcode: 0x04,
            name: "INC B",
            cycles: 1,
            size: 1,
            flags: &[FlagBits::H],
        });
        all_instructions.insert(0x05, Instruction {
            opcode: 0x05,
            name: "DEC B",
            cycles: 1,
            size: 1,
            flags: &[FlagBits::Z, FlagBits::H],
        });
        all_instructions.insert(0x06, Instruction {
            opcode: 0x06,
            name: "LD B D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x07, Instruction {
            opcode: 0x07,
            name: "RLCA",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x08, Instruction {
            opcode: 0x08,
            name: "LD (A16) SP",
            cycles: 5,
            size: 3,
            flags: &[FlagBits::C],
        });
        all_instructions.insert(0x09, Instruction {
            opcode: 0x09,
            name: "ADD HL BC",
            cycles: 2,
            size: 1,
            flags: &[FlagBits::C],
        });
        all_instructions.insert(0x0A, Instruction {
            opcode: 0x0A,
            name: "LD A (BC)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0B, Instruction {
            opcode: 0x0B,
            name: "DEC BC",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0C, Instruction {
            opcode: 0x0C,
            name: "INC C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0D, Instruction {
            opcode: 0x0D,
            name: "DEC C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0E, Instruction {
            opcode: 0x0E,
            name: "LD C D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x0F, Instruction {
            opcode: 0x0F,
            name: "RRCA",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x10, Instruction {
            opcode: 0x10,
            name: "STOP 0",
            cycles: 1,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x11, Instruction {
            opcode: 0x11,
            name: "LD DE D16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0x12, Instruction {
            opcode: 0x12,
            name: "LD (DE) A",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x13, Instruction {
            opcode: 0x13,
            name: "INC DE",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x14, Instruction {
            opcode: 0x14,
            name: "INC D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x15, Instruction {
            opcode: 0x15,
            name: "DEC D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x16, Instruction {
            opcode: 0x16,
            name: "LD D D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x17, Instruction {
            opcode: 0x17,
            name: "RLA",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x18, Instruction {
            opcode: 0x18,
            name: "JR R8",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x19, Instruction {
            opcode: 0x19,
            name: "ADD HL DE",
            cycles: 2,
            size: 1,
            flags: &[],
        });

        all_instructions.insert(0x1A, Instruction {
            opcode: 0x1A,
            name: "LD A (DE)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1B, Instruction {
            opcode: 0x1B,
            name: "DEC DE",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1C, Instruction {
            opcode: 0x1C,
            name: "INC E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1D, Instruction {
            opcode: 0x1D,
            name: "DEC E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1E, Instruction {
            opcode: 0x1E,
            name: "LD E D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x1F, Instruction {
            opcode: 0x1F,
            name: "RRA",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x20, Instruction {
            opcode: 0x20,
            name: "JR NZ R8",
            cycles: 3, //tbd how to handle or 2 if not jump, maybe second cycle as option and handle in func
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x21, Instruction {
            opcode: 0x21,
            name: "LD HL D16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0x22, Instruction {
            opcode: 0x22,
            name: "LD (HL+) A",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x23, Instruction {
            opcode: 0x23,
            name: "INC HL",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x24, Instruction {
            opcode: 0x24,
            name: "INC H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x25, Instruction {
            opcode: 0x25,
            name: "DEC H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x26, Instruction {
            opcode: 0x26,
            name: "LD H D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x27, Instruction {
            opcode: 0x27,
            name: "DAA",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x28, Instruction {
            opcode: 0x28,
            name: "JR Z R8",
            cycles: 12, // or 8 if condition not met
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x29, Instruction {
            opcode: 0x29,
            name: "ADD HL HL",
            cycles: 8,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2A, Instruction {
            opcode: 0x2A,
            name: "LD A (HL+)",
            cycles: 8,
            size: 1,
            flags: &[],
        });

        all_instructions.insert(0x2B, Instruction {
            opcode: 0x2B,
            name: "DEC HL",
            cycles: 8,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2C, Instruction {
            opcode: 0x2C,
            name: "INC L",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2D, Instruction {
            opcode: 0x2D,
            name: "DEC L",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2E, Instruction {
            opcode: 0x2E,
            name: "LD L D8",
            cycles: 8,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2F, Instruction {
            opcode: 0x2F,
            name: "CPL",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x30, Instruction {
            opcode: 0x30,
            name: "JR NC R8",
            cycles: 12,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x31, Instruction {
            opcode: 0x31,
            name: "LD SP D16",
            cycles: 12,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0x32, Instruction {
            opcode: 0x32,
            name: "LD HLD A",
            cycles: 8,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x33, Instruction {
            opcode: 0x33,
            name: "INC SP",
            cycles: 8,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x34, Instruction {
            opcode: 0x34,
            name: "INC HL",
            cycles: 12,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x35, Instruction {
            opcode: 0x35,
            name: "DEC HL",
            cycles: 12,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x36, Instruction {
            opcode: 0x36,
            name: "LD HL D8",
            cycles: 12,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x37, Instruction {
            opcode: 0x37,
            name: "SCF",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x38, Instruction {
            opcode: 0x38,
            name: "JR C R8",
            cycles: 12,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x39, Instruction {
            opcode: 0x39,
            name: "ADD HL SP",
            cycles: 8,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3A, Instruction {
            opcode: 0x3A,
            name: "LD A (DE)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3B, Instruction {
            opcode: 0x3B,
            name: "DEC SP",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3C, Instruction {
            opcode: 0x3C,
            name: "INC A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3D, Instruction {
            opcode: 0x3D,
            name: "DEC A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3E, Instruction {
            opcode: 0x3E,
            name: "LD A d8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x3F, Instruction {
            opcode: 0x3F,
            name: "CCF",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x40, Instruction {
            opcode: 0x40,
            name: "LD B B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x41, Instruction {
            opcode: 0x41,
            name: "LD B C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x42, Instruction {
            opcode: 0x42,
            name: "LD B D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x43, Instruction {
            opcode: 0x43,
            name: "LD B E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x44, Instruction {
            opcode: 0x44,
            name: "LD B H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x45, Instruction {
            opcode: 0x45,
            name: "LD B L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x46, Instruction {
            opcode: 0x46,
            name: "LD B (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x47, Instruction {
            opcode: 0x47,
            name: "LD B A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x48, Instruction {
            opcode: 0x48,
            name: "LD C B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x49, Instruction {
            opcode: 0x49,
            name: "LD C C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4A, Instruction {
            opcode: 0x4A,
            name: "LD C D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4B, Instruction {
            opcode: 0x4B,
            name: "LD C E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4C, Instruction {
            opcode: 0x4C,
            name: "LD C H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4D, Instruction {
            opcode: 0x4D,
            name: "LD C L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4E, Instruction {
            opcode: 0x4E,
            name: "LD C (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4F, Instruction {
            opcode: 0x4F,
            name: "LD C A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x50, Instruction {
            opcode: 0x50,
            name: "LD D B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x51, Instruction {
            opcode: 0x51,
            name: "LD D C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x52, Instruction {
            opcode: 0x52,
            name: "LD D D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x53, Instruction {
            opcode: 0x53,
            name: "LD D E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x54, Instruction {
            opcode: 0x54,
            name: "LD D H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x55, Instruction {
            opcode: 0x55,
            name: "LD D L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x56, Instruction {
            opcode: 0x56,
            name: "LD D (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x57, Instruction {
            opcode: 0x57,
            name: "LD D A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x58, Instruction {
            opcode: 0x58,
            name: "LD E B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x59, Instruction {
            opcode: 0x59,
            name: "LD E C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5A, Instruction {
            opcode: 0x5A,
            name: "LD E D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5B, Instruction {
            opcode: 0x5B,
            name: "LD E E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5C, Instruction {
            opcode: 0x5C,
            name: "LD E H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5D, Instruction {
            opcode: 0x5D,
            name: "LD E L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5E, Instruction {
            opcode: 0x5E,
            name: "LD E (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5F, Instruction {
            opcode: 0x5F,
            name: "LD E A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x60, Instruction {
            opcode: 0x60,
            name: "LD H B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x61, Instruction {
            opcode: 0x61,
            name: "LD H C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x62, Instruction {
            opcode: 0x62,
            name: "LD H D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x63, Instruction {
            opcode: 0x63,
            name: "LD H E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x64, Instruction {
            opcode: 0x64,
            name: "LD H H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x65, Instruction {
            opcode: 0x65,
            name: "LD H L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x66, Instruction {
            opcode: 0x66,
            name: "LD H (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x67, Instruction {
            opcode: 0x67,
            name: "LD H A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x68, Instruction {
            opcode: 0x68,
            name: "LD L B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x69, Instruction {
            opcode: 0x69,
            name: "LD L C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6A, Instruction {
            opcode: 0x6A,
            name: "LD L D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6B, Instruction {
            opcode: 0x6B,
            name: "LD L E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6C, Instruction {
            opcode: 0x6C,
            name: "LD L H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6D, Instruction {
            opcode: 0x6D,
            name: "LD L L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6E, Instruction {
            opcode: 0x6E,
            name: "LD L (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6F, Instruction {
            opcode: 0x6F,
            name: "LD L A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x70, Instruction {
            opcode: 0x70,
            name: "LD (HL) B",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x71, Instruction {
            opcode: 0x71,
            name: "LD (HL)  C",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x72, Instruction {
            opcode: 0x72,
            name: "LD (HL) D",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x73, Instruction {
            opcode: 0x73,
            name: "LD (HL) E",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x74, Instruction {
            opcode: 0x74,
            name: "LD (HL) H",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x75, Instruction {
            opcode: 0x75,
            name: "LD (HL) L",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x76, Instruction {
            opcode: 0x76,
            name: "HALT",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x77, Instruction {
            opcode: 0x77,
            name: "LD (HL) A",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x78, Instruction {
            opcode: 0x78,
            name: "LD A B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x79, Instruction {
            opcode: 0x79,
            name: "LD A C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7A, Instruction {
            opcode: 0x7A,
            name: "LD A D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7B, Instruction {
            opcode: 0x7B,
            name: "LD A E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7C, Instruction {
            opcode: 0x7C,
            name: "LD A H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7D, Instruction {
            opcode: 0x7D,
            name: "LD A L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7E, Instruction {
            opcode: 0x7E,
            name: "LD A (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7F, Instruction {
            opcode: 0x7F,
            name: "LD A A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x80, Instruction {
            opcode: 0x80,
            name: "ADD A B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x81, Instruction {
            opcode: 0x81,
            name: "ADD A C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x82, Instruction {
            opcode: 0x82,
            name: "ADD A D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x83, Instruction {
            opcode: 0x83,
            name: "ADD A E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x84, Instruction {
            opcode: 0x84,
            name: "ADD A H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x85, Instruction {
            opcode: 0x85,
            name: "ADD A L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x86, Instruction {
            opcode: 0x86,
            name: "ADD A (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x87, Instruction {
            opcode: 0x87,
            name: "ADD A, A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x88, Instruction {
            opcode: 0x88,
            name: "ADC A B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x89, Instruction {
            opcode: 0x89,
            name: "ADC A C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8A, Instruction {
            opcode: 0x8A,
            name: "ADC A D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8B, Instruction {
            opcode: 0x8B,
            name: "ADC A E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8C, Instruction {
            opcode: 0x8C,
            name: "ADC A H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8D, Instruction {
            opcode: 0x8D,
            name: "ADC A L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8E, Instruction {
            opcode: 0x8E,
            name: "ADC A (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8F, Instruction {
            opcode: 0x8F,
            name: "ADC A A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x90, Instruction {
            opcode: 0x90,
            name: "UNIMPLEMENTED_90",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x91, Instruction {
            opcode: 0x91,
            name: "UNIMPLEMENTED_91",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x92, Instruction {
            opcode: 0x92,
            name: "UNIMPLEMENTED_92",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x93, Instruction {
            opcode: 0x93,
            name: "UNIMPLEMENTED_93",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x94, Instruction {
            opcode: 0x94,
            name: "UNIMPLEMENTED_94",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x95, Instruction {
            opcode: 0x95,
            name: "UNIMPLEMENTED_95",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x96, Instruction {
            opcode: 0x96,
            name: "UNIMPLEMENTED_96",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x97, Instruction {
            opcode: 0x97,
            name: "UNIMPLEMENTED_97",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x98, Instruction {
            opcode: 0x98,
            name: "UNIMPLEMENTED_98",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x99, Instruction {
            opcode: 0x99,
            name: "UNIMPLEMENTED_99",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9A, Instruction {
            opcode: 0x9A,
            name: "UNIMPLEMENTED_9A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9B, Instruction {
            opcode: 0x9B,
            name: "UNIMPLEMENTED_9B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9C, Instruction {
            opcode: 0x9C,
            name: "UNIMPLEMENTED_9C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9D, Instruction {
            opcode: 0x9D,
            name: "UNIMPLEMENTED_9D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9E, Instruction {
            opcode: 0x9E,
            name: "UNIMPLEMENTED_9E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9F, Instruction {
            opcode: 0x9F,
            name: "UNIMPLEMENTED_9F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA0, Instruction {
            opcode: 0xA0,
            name: "UNIMPLEMENTED_A0",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA1, Instruction {
            opcode: 0xA1,
            name: "UNIMPLEMENTED_A1",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA2, Instruction {
            opcode: 0xA2,
            name: "UNIMPLEMENTED_A2",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA3, Instruction {
            opcode: 0xA3,
            name: "UNIMPLEMENTED_A3",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA4, Instruction {
            opcode: 0xA4,
            name: "UNIMPLEMENTED_A4",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA5, Instruction {
            opcode: 0xA5,
            name: "UNIMPLEMENTED_A5",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA6, Instruction {
            opcode: 0xA6,
            name: "UNIMPLEMENTED_A6",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA7, Instruction {
            opcode: 0xA7,
            name: "UNIMPLEMENTED_A7",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA8, Instruction {
            opcode: 0xA8,
            name: "UNIMPLEMENTED_A8",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA9, Instruction {
            opcode: 0xA9,
            name: "UNIMPLEMENTED_A9",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAA, Instruction {
            opcode: 0xAA,
            name: "UNIMPLEMENTED_AA",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAB, Instruction {
            opcode: 0xAB,
            name: "UNIMPLEMENTED_AB",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAC, Instruction {
            opcode: 0xAC,
            name: "UNIMPLEMENTED_AC",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAD, Instruction {
            opcode: 0xAD,
            name: "UNIMPLEMENTED_AD",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAE, Instruction {
            opcode: 0xAE,
            name: "UNIMPLEMENTED_AE",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAF, Instruction {
            opcode: 0xAF,
            name: "UNIMPLEMENTED_AF",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB0, Instruction {
            opcode: 0xB0,
            name: "UNIMPLEMENTED_B0",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB1, Instruction {
            opcode: 0xB1,
            name: "UNIMPLEMENTED_B1",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB2, Instruction {
            opcode: 0xB2,
            name: "UNIMPLEMENTED_B2",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB3, Instruction {
            opcode: 0xB3,
            name: "UNIMPLEMENTED_B3",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB4, Instruction {
            opcode: 0xB4,
            name: "UNIMPLEMENTED_B4",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB5, Instruction {
            opcode: 0xB5,
            name: "UNIMPLEMENTED_B5",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB6, Instruction {
            opcode: 0xB6,
            name: "UNIMPLEMENTED_B6",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB7, Instruction {
            opcode: 0xB7,
            name: "UNIMPLEMENTED_B7",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB8, Instruction {
            opcode: 0xB8,
            name: "UNIMPLEMENTED_B8",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB9, Instruction {
            opcode: 0xB9,
            name: "UNIMPLEMENTED_B9",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBA, Instruction {
            opcode: 0xBA,
            name: "UNIMPLEMENTED_BA",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBB, Instruction {
            opcode: 0xBB,
            name: "UNIMPLEMENTED_BB",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBC, Instruction {
            opcode: 0xBC,
            name: "UNIMPLEMENTED_BC",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBD, Instruction {
            opcode: 0xBD,
            name: "UNIMPLEMENTED_BD",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBE, Instruction {
            opcode: 0xBE,
            name: "UNIMPLEMENTED_BE",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBF, Instruction {
            opcode: 0xBF,
            name: "UNIMPLEMENTED_BF",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC0, Instruction {
            opcode: 0xC0,
            name: "UNIMPLEMENTED_C0",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC1, Instruction {
            opcode: 0xC1,
            name: "UNIMPLEMENTED_C1",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC2, Instruction {
            opcode: 0xC2,
            name: "UNIMPLEMENTED_C2",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC3, Instruction {
            opcode: 0xC3,
            name: "UNIMPLEMENTED_C3",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC4, Instruction {
            opcode: 0xC4,
            name: "UNIMPLEMENTED_C4",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC5, Instruction {
            opcode: 0xC5,
            name: "UNIMPLEMENTED_C5",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC6, Instruction {
            opcode: 0xC6,
            name: "UNIMPLEMENTED_C6",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC7, Instruction {
            opcode: 0xC7,
            name: "UNIMPLEMENTED_C7",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC8, Instruction {
            opcode: 0xC8,
            name: "UNIMPLEMENTED_C8",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC9, Instruction {
            opcode: 0xC9,
            name: "UNIMPLEMENTED_C9",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCA, Instruction {
            opcode: 0xCA,
            name: "UNIMPLEMENTED_CA",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCB, Instruction {
            opcode: 0xCB,
            name: "UNIMPLEMENTED_CB",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCC, Instruction {
            opcode: 0xCC,
            name: "UNIMPLEMENTED_CC",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCD, Instruction {
            opcode: 0xCD,
            name: "UNIMPLEMENTED_CD",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCE, Instruction {
            opcode: 0xCE,
            name: "UNIMPLEMENTED_CE",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCF, Instruction {
            opcode: 0xCF,
            name: "UNIMPLEMENTED_CF",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD0, Instruction {
            opcode: 0xD0,
            name: "UNIMPLEMENTED_D0",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD1, Instruction {
            opcode: 0xD1,
            name: "UNIMPLEMENTED_D1",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD2, Instruction {
            opcode: 0xD2,
            name: "UNIMPLEMENTED_D2",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD3, Instruction {
            opcode: 0xD3,
            name: "UNIMPLEMENTED_D3",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD4, Instruction {
            opcode: 0xD4,
            name: "UNIMPLEMENTED_D4",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD5, Instruction {
            opcode: 0xD5,
            name: "UNIMPLEMENTED_D5",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD6, Instruction {
            opcode: 0xD6,
            name: "UNIMPLEMENTED_D6",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD7, Instruction {
            opcode: 0xD7,
            name: "UNIMPLEMENTED_D7",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD8, Instruction {
            opcode: 0xD8,
            name: "UNIMPLEMENTED_D8",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD9, Instruction {
            opcode: 0xD9,
            name: "UNIMPLEMENTED_D9",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDA, Instruction {
            opcode: 0xDA,
            name: "UNIMPLEMENTED_DA",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDB, Instruction {
            opcode: 0xDB,
            name: "UNIMPLEMENTED_DB",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDC, Instruction {
            opcode: 0xDC,
            name: "UNIMPLEMENTED_DC",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDD, Instruction {
            opcode: 0xDD,
            name: "UNIMPLEMENTED_DD",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDE, Instruction {
            opcode: 0xDE,
            name: "UNIMPLEMENTED_DE",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDF, Instruction {
            opcode: 0xDF,
            name: "UNIMPLEMENTED_DF",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE0, Instruction {
            opcode: 0xE0,
            name: "UNIMPLEMENTED_E0",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE1, Instruction {
            opcode: 0xE1,
            name: "UNIMPLEMENTED_E1",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE2, Instruction {
            opcode: 0xE2,
            name: "UNIMPLEMENTED_E2",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE3, Instruction {
            opcode: 0xE3,
            name: "UNIMPLEMENTED_E3",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE4, Instruction {
            opcode: 0xE4,
            name: "UNIMPLEMENTED_E4",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE5, Instruction {
            opcode: 0xE5,
            name: "UNIMPLEMENTED_E5",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE6, Instruction {
            opcode: 0xE6,
            name: "UNIMPLEMENTED_E6",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE7, Instruction {
            opcode: 0xE7,
            name: "UNIMPLEMENTED_E7",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE8, Instruction {
            opcode: 0xE8,
            name: "UNIMPLEMENTED_E8",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE9, Instruction {
            opcode: 0xE9,
            name: "UNIMPLEMENTED_E9",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xEA, Instruction {
            opcode: 0xEA,
            name: "UNIMPLEMENTED_EA",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xEB, Instruction {
            opcode: 0xEB,
            name: "UNIMPLEMENTED_EB",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xEC, Instruction {
            opcode: 0xEC,
            name: "UNIMPLEMENTED_EC",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xED, Instruction {
            opcode: 0xED,
            name: "UNIMPLEMENTED_ED",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xEE, Instruction {
            opcode: 0xEE,
            name: "UNIMPLEMENTED_EE",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xEF, Instruction {
            opcode: 0xEF,
            name: "UNIMPLEMENTED_EF",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF0, Instruction {
            opcode: 0xF0,
            name: "UNIMPLEMENTED_F0",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF1, Instruction {
            opcode: 0xF1,
            name: "UNIMPLEMENTED_F1",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF2, Instruction {
            opcode: 0xF2,
            name: "UNIMPLEMENTED_F2",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF3, Instruction {
            opcode: 0xF3,
            name: "UNIMPLEMENTED_F3",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF4, Instruction {
            opcode: 0xF4,
            name: "UNIMPLEMENTED_F4",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF5, Instruction {
            opcode: 0xF5,
            name: "UNIMPLEMENTED_F5",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF6, Instruction {
            opcode: 0xF6,
            name: "UNIMPLEMENTED_F6",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF7, Instruction {
            opcode: 0xF7,
            name: "UNIMPLEMENTED_F7",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF8, Instruction {
            opcode: 0xF8,
            name: "UNIMPLEMENTED_F8",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF9, Instruction {
            opcode: 0xF9,
            name: "UNIMPLEMENTED_F9",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFA, Instruction {
            opcode: 0xFA,
            name: "UNIMPLEMENTED_FA",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFB, Instruction {
            opcode: 0xFB,
            name: "UNIMPLEMENTED_FB",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFC, Instruction {
            opcode: 0xFC,
            name: "UNIMPLEMENTED_FC",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFD, Instruction {
            opcode: 0xFD,
            name: "UNIMPLEMENTED_FD",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFE, Instruction {
            opcode: 0xFE,
            name: "UNIMPLEMENTED_FE",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFF, Instruction {
            opcode: 0xFF,
            name: "UNIMPLEMENTED_FF",
            cycles: 0,
            size: 1,
            flags: &[],
        });
    
        all_instructions
    }
    pub fn setup_cb_inst() -> HashMap<u8, Instruction> {
        // https://meganesu.github.io/generate-gb-opcodes/
        let mut all_instructions = HashMap::new();
     
        all_instructions.insert(0x00, Instruction {
            opcode: 0x00,
            name: "RLC B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x01, Instruction {
            opcode: 0x01,
            name: "RLC C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x02, Instruction {
            opcode: 0x02,
            name: "RLC D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x03, Instruction {
            opcode: 0x03,
            name: "RLC E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x04, Instruction {
            opcode: 0x04,
            name: "RLC H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x05, Instruction {
            opcode: 0x05,
            name: "RLC L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x06, Instruction {
            opcode: 0x06,
            name: "RLC (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x07, Instruction {
            opcode: 0x07,
            name: "RLC A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x08, Instruction {
            opcode: 0x08,
            name: "RRC B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x09, Instruction {
            opcode: 0x09,
            name: "RRC C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x0A, Instruction {
            opcode: 0x0A,
            name: "RRC D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x0B, Instruction {
            opcode: 0x0B,
            name: "RRC E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x0C, Instruction {
            opcode: 0x0C,
            name: "RRC H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x0D, Instruction {
            opcode: 0x0D,
            name: "RRC L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x0E, Instruction {
            opcode: 0x0E,
            name: "RRC (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x0F, Instruction {
            opcode: 0x0F,
            name: "RRC A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x10, Instruction {
            opcode: 0x10,
            name: "RL B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x11, Instruction {
            opcode: 0x11,
            name: "RL C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x12, Instruction {
            opcode: 0x12,
            name: "RL D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x13, Instruction {
            opcode: 0x13,
            name: "RL E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x14, Instruction {
            opcode: 0x14,
            name: "RL H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x15, Instruction {
            opcode: 0x15,
            name: "RL L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x16, Instruction {
            opcode: 0x16,
            name: "RL (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x17, Instruction {
            opcode: 0x17,
            name: "RL A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x18, Instruction {
            opcode: 0x18,
            name: "RR B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x19, Instruction {
            opcode: 0x19,
            name: "RR C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x1A, Instruction {
            opcode: 0x1A,
            name: "RR D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x1B, Instruction {
            opcode: 0x1B,
            name: "RR E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x1C, Instruction {
            opcode: 0x1C,
            name: "RR H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x1D, Instruction {
            opcode: 0x1D,
            name: "RR L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x1E, Instruction {
            opcode: 0x1E,
            name: "RR (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x1F, Instruction {
            opcode: 0x1F,
            name: "RR A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x20, Instruction {
            opcode: 0x20,
            name: "SLA B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x21, Instruction {
            opcode: 0x21,
            name: "SLA C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x22, Instruction {
            opcode: 0x22,
            name: "SLA D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x23, Instruction {
            opcode: 0x23,
            name: "SLA E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x24, Instruction {
            opcode: 0x24,
            name: "SLA H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x25, Instruction {
            opcode: 0x25,
            name: "SLA L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x26, Instruction {
            opcode: 0x26,
            name: "SLA (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x27, Instruction {
            opcode: 0x27,
            name: "SLA A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x28, Instruction {
            opcode: 0x28,
            name: "SRA B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x29, Instruction {
            opcode: 0x29,
            name: "SRA C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2A, Instruction {
            opcode: 0x2A,
            name: "SRA D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2B, Instruction {
            opcode: 0x2B,
            name: "SRA E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2C, Instruction {
            opcode: 0x2C,
            name: "SRA H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2D, Instruction {
            opcode: 0x2D,
            name: "SRA L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2E, Instruction {
            opcode: 0x2E,
            name: "SRA (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2F, Instruction {
            opcode: 0x2F,
            name: "SRA A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x30, Instruction {
            opcode: 0x30,
            name: "SWAP B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x31, Instruction {
            opcode: 0x31,
            name: "SWAP C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x32, Instruction {
            opcode: 0x32,
            name: "SWAP D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x33, Instruction {
            opcode: 0x33,
            name: "SWAP E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x34, Instruction {
            opcode: 0x34,
            name: "SWAP H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x35, Instruction {
            opcode: 0x35,
            name: "SWAP L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x36, Instruction {
            opcode: 0x36,
            name: "SWAP (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x37, Instruction {
            opcode: 0x37,
            name: "SWAP A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x38, Instruction {
            opcode: 0x38,
            name: "SRL B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x39, Instruction {
            opcode: 0x39,
            name: "SRL C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x3A, Instruction {
            opcode: 0x3A,
            name: "SRL D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x3B, Instruction {
            opcode: 0x3B,
            name: "SRL E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x3C, Instruction {
            opcode: 0x3C,
            name: "SRL H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x3D, Instruction {
            opcode: 0x3D,
            name: "SRL L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x3E, Instruction {
            opcode: 0x3E,
            name: "SRL (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x3F, Instruction {
            opcode: 0x3F,
            name: "SRL A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x40, Instruction {
            opcode: 0x40,
            name: "BIT 0 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x41, Instruction {
            opcode: 0x41,
            name: "BIT 0 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x42, Instruction {
            opcode: 0x42,
            name: "BIT 0 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x43, Instruction {
            opcode: 0x43,
            name: "BIT 0 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x44, Instruction {
            opcode: 0x44,
            name: "BIT 0 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x45, Instruction {
            opcode: 0x45,
            name: "BIT 0 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x46, Instruction {
            opcode: 0x46,
            name: "BIT 0 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x47, Instruction {
            opcode: 0x47,
            name: "BIT 0 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x48, Instruction {
            opcode: 0x48,
            name: "BIT 1 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x49, Instruction {
            opcode: 0x49,
            name: "BIT 1 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x4A, Instruction {
            opcode: 0x4A,
            name: "BIT 1 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x4B, Instruction {
            opcode: 0x4B,
            name: "BIT 1 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x4C, Instruction {
            opcode: 0x4C,
            name: "BIT 1 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x4D, Instruction {
            opcode: 0x4D,
            name: "BIT 1 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x4E, Instruction {
            opcode: 0x4E,
            name: "BIT 1 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x4F, Instruction {
            opcode: 0x4F,
            name: "BIT 1 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x50, Instruction {
            opcode: 0x50,
            name: "BIT 2 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x51, Instruction {
            opcode: 0x51,
            name: "BIT 2 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x52, Instruction {
            opcode: 0x52,
            name: "BIT 2 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x53, Instruction {
            opcode: 0x53,
            name: "BIT 2 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x54, Instruction {
            opcode: 0x54,
            name: "BIT 2 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x55, Instruction {
            opcode: 0x55,
            name: "BIT 2 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x56, Instruction {
            opcode: 0x56,
            name: "BIT 2 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x57, Instruction {
            opcode: 0x57,
            name: "BIT 2 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x58, Instruction {
            opcode: 0x58,
            name: "BIT 3 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x59, Instruction {
            opcode: 0x59,
            name: "BIT 3 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x5A, Instruction {
            opcode: 0x5A,
            name: "BIT 3 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x5B, Instruction {
            opcode: 0x5B,
            name: "BIT 3 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x5C, Instruction {
            opcode: 0x5C,
            name: "BIT 3 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x5D, Instruction {
            opcode: 0x5D,
            name: "BIT 3 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x5E, Instruction {
            opcode: 0x5E,
            name: "BIT 3 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x5F, Instruction {
            opcode: 0x5F,
            name: "BIT 3 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x60, Instruction {
            opcode: 0x60,
            name: "BIT 4 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x61, Instruction {
            opcode: 0x61,
            name: "BIT 4 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x62, Instruction {
            opcode: 0x62,
            name: "BIT 4 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x63, Instruction {
            opcode: 0x63,
            name: "BIT 4 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x64, Instruction {
            opcode: 0x64,
            name: "BIT 4 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x65, Instruction {
            opcode: 0x65,
            name: "BIT 4 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x66, Instruction {
            opcode: 0x66,
            name: "BIT 4 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x67, Instruction {
            opcode: 0x67,
            name: "BIT 4 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x68, Instruction {
            opcode: 0x68,
            name: "BIT 5 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x69, Instruction {
            opcode: 0x69,
            name: "BIT 5 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x6A, Instruction {
            opcode: 0x6A,
            name: "BIT 5 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x6B, Instruction {
            opcode: 0x6B,
            name: "BIT 5 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x6C, Instruction {
            opcode: 0x6C,
            name: "BIT 5 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x6D, Instruction {
            opcode: 0x6D,
            name: "BIT 5 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x6E, Instruction {
            opcode: 0x6E,
            name: "BIT 5 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x6F, Instruction {
            opcode: 0x6F,
            name: "BIT 5 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x70, Instruction {
            opcode: 0x70,
            name: "BIT 6 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x71, Instruction {
            opcode: 0x71,
            name: "BIT 6 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x72, Instruction {
            opcode: 0x72,
            name: "BIT 6 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x73, Instruction {
            opcode: 0x73,
            name: "BIT 6 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x74, Instruction {
            opcode: 0x74,
            name: "BIT 6 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x75, Instruction {
            opcode: 0x75,
            name: "BIT 6 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x76, Instruction {
            opcode: 0x76,
            name: "BIT 6 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x77, Instruction {
            opcode: 0x77,
            name: "BIT 6 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x78, Instruction {
            opcode: 0x78,
            name: "BIT 7 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x79, Instruction {
            opcode: 0x79,
            name: "BIT 7 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x7A, Instruction {
            opcode: 0x7A,
            name: "BIT 7 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x7B, Instruction {
            opcode: 0x7B,
            name: "BIT 7 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x7C, Instruction {
            opcode: 0x7C,
            name: "BIT 7 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x7D, Instruction {
            opcode: 0x7D,
            name: "BIT 7 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x7E, Instruction {
            opcode: 0x7E,
            name: "BIT 7 (HL)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x7F, Instruction {
            opcode: 0x7F,
            name: "BIT 7 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x80, Instruction {
            opcode: 0x80,
            name: "RES 0 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x81, Instruction {
            opcode: 0x81,
            name: "RES 0 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x82, Instruction {
            opcode: 0x82,
            name: "RES 0 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x83, Instruction {
            opcode: 0x83,
            name: "RES 0 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x84, Instruction {
            opcode: 0x84,
            name: "RES 0 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x85, Instruction {
            opcode: 0x85,
            name: "RES 0 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x86, Instruction {
            opcode: 0x86,
            name: "RES 0 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x87, Instruction {
            opcode: 0x87,
            name: "RES 0 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x88, Instruction {
            opcode: 0x88,
            name: "RES 1 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x89, Instruction {
            opcode: 0x89,
            name: "RES 1 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x8A, Instruction {
            opcode: 0x8A,
            name: "RES 1 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x8B, Instruction {
            opcode: 0x8B,
            name: "RES 1 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x8C, Instruction {
            opcode: 0x8C,
            name: "RES 1 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x8D, Instruction {
            opcode: 0x8D,
            name: "RES 1 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x8E, Instruction {
            opcode: 0x8E,
            name: "RES 1 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x8F, Instruction {
            opcode: 0x8F,
            name: "RES 1 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x90, Instruction {
            opcode: 0x90,
            name: "RES 2 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x91, Instruction {
            opcode: 0x91,
            name: "RES 2 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x92, Instruction {
            opcode: 0x92,
            name: "RES 2 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x93, Instruction {
            opcode: 0x93,
            name: "RES 2 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x94, Instruction {
            opcode: 0x94,
            name: "RES 2 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x95, Instruction {
            opcode: 0x95,
            name: "RES 2 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x96, Instruction {
            opcode: 0x96,
            name: "RES 2 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x97, Instruction {
            opcode: 0x97,
            name: "RES 2 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x98, Instruction {
            opcode: 0x98,
            name: "RES 3 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x99, Instruction {
            opcode: 0x99,
            name: "RES 3 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x9A, Instruction {
            opcode: 0x9A,
            name: "RES 3 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x9B, Instruction {
            opcode: 0x9B,
            name: "RES 3 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x9C, Instruction {
            opcode: 0x9C,
            name: "RES 3 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x9D, Instruction {
            opcode: 0x9D,
            name: "RES 3 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x9E, Instruction {
            opcode: 0x9E,
            name: "RES 3 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x9F, Instruction {
            opcode: 0x9F,
            name: "RES 3 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA0, Instruction {
            opcode: 0xA0,
            name: "RES 4 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA1, Instruction {
            opcode: 0xA1,
            name: "RES 4 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA2, Instruction {
            opcode: 0xA2,
            name: "RES 4 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA3, Instruction {
            opcode: 0xA3,
            name: "RES 4 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA4, Instruction {
            opcode: 0xA4,
            name: "RES 4 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA5, Instruction {
            opcode: 0xA5,
            name: "RES 4 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA6, Instruction {
            opcode: 0xA6,
            name: "RES 4 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA7, Instruction {
            opcode: 0xA7,
            name: "RES 4 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA8, Instruction {
            opcode: 0xA8,
            name: "RES 5 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xA9, Instruction {
            opcode: 0xA9,
            name: "RES 5 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xAA, Instruction {
            opcode: 0xAA,
            name: "RES 5 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xAB, Instruction {
            opcode: 0xAB,
            name: "RES 5 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xAC, Instruction {
            opcode: 0xAC,
            name: "RES 5 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xAD, Instruction {
            opcode: 0xAD,
            name: "RES 5 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xAE, Instruction {
            opcode: 0xAE,
            name: "RES 5 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xAF, Instruction {
            opcode: 0xAF,
            name: "RES 5 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB0, Instruction {
            opcode: 0xB0,
            name: "RES 6 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB1, Instruction {
            opcode: 0xB1,
            name: "RES 6 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB2, Instruction {
            opcode: 0xB2,
            name: "RES 6 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB3, Instruction {
            opcode: 0xB3,
            name: "RES 6 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB4, Instruction {
            opcode: 0xB4,
            name: "RES 6 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB5, Instruction {
            opcode: 0xB5,
            name: "RES 6 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB6, Instruction {
            opcode: 0xB6,
            name: "RES 6 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB7, Instruction {
            opcode: 0xB7,
            name: "RES 6 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB8, Instruction {
            opcode: 0xB8,
            name: "RES 7 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xB9, Instruction {
            opcode: 0xB9,
            name: "RES 7 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xBA, Instruction {
            opcode: 0xBA,
            name: "RES 7 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xBB, Instruction {
            opcode: 0xBB,
            name: "RES 7 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xBC, Instruction {
            opcode: 0xBC,
            name: "RES 7 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xBD, Instruction {
            opcode: 0xBD,
            name: "RES 7 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xBE, Instruction {
            opcode: 0xBE,
            name: "RES 7 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xBF, Instruction {
            opcode: 0xBF,
            name: "RES 7 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC0, Instruction {
            opcode: 0xC0,
            name: "SET 0 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC1, Instruction {
            opcode: 0xC1,
            name: "SET 0 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC2, Instruction {
            opcode: 0xC2,
            name: "SET 0 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC3, Instruction {
            opcode: 0xC3,
            name: "SET 0 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC4, Instruction {
            opcode: 0xC4,
            name: "SET 0 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC5, Instruction {
            opcode: 0xC5,
            name: "SET 0 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC6, Instruction {
            opcode: 0xC6,
            name: "SET 0 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC7, Instruction {
            opcode: 0xC7,
            name: "SET 0 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC8, Instruction {
            opcode: 0xC8,
            name: "SET 1 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC9, Instruction {
            opcode: 0xC9,
            name: "SET 1 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xCA, Instruction {
            opcode: 0xCA,
            name: "SET 1 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xCB, Instruction {
            opcode: 0xCB,
            name: "SET 1 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xCC, Instruction {
            opcode: 0xCC,
            name: "SET 1 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xCD, Instruction {
            opcode: 0xCD,
            name: "SET 1 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xCE, Instruction {
            opcode: 0xCE,
            name: "SET 1 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xCF, Instruction {
            opcode: 0xCF,
            name: "SET 1 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD0, Instruction {
            opcode: 0xD0,
            name: "SET 2 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD1, Instruction {
            opcode: 0xD1,
            name: "SET 2 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD2, Instruction {
            opcode: 0xD2,
            name: "SET 2 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD3, Instruction {
            opcode: 0xD3,
            name: "SET 2 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD4, Instruction {
            opcode: 0xD4,
            name: "SET 2 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD5, Instruction {
            opcode: 0xD5,
            name: "SET 2 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD6, Instruction {
            opcode: 0xD6,
            name: "SET 2 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD7, Instruction {
            opcode: 0xD7,
            name: "SET 2 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD8, Instruction {
            opcode: 0xD8,
            name: "SET 3 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD9, Instruction {
            opcode: 0xD9,
            name: "SET 3 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xDA, Instruction {
            opcode: 0xDA,
            name: "SET 3 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xDB, Instruction {
            opcode: 0xDB,
            name: "SET 3 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xDC, Instruction {
            opcode: 0xDC,
            name: "SET 3 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xDD, Instruction {
            opcode: 0xDD,
            name: "SET 3 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xDE, Instruction {
            opcode: 0xDE,
            name: "SET 3 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xDF, Instruction {
            opcode: 0xDF,
            name: "SET 3 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE0, Instruction {
            opcode: 0xE0,
            name: "SET 4 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE1, Instruction {
            opcode: 0xE1,
            name: "SET 4 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE2, Instruction {
            opcode: 0xE2,
            name: "SET 4 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE3, Instruction {
            opcode: 0xE3,
            name: "SET 4 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE4, Instruction {
            opcode: 0xE4,
            name: "SET 4 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE5, Instruction {
            opcode: 0xE5,
            name: "SET 4 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE6, Instruction {
            opcode: 0xE6,
            name: "SET 4 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE7, Instruction {
            opcode: 0xE7,
            name: "SET 4 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE8, Instruction {
            opcode: 0xE8,
            name: "SET 5 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE9, Instruction {
            opcode: 0xE9,
            name: "SET 5 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xEA, Instruction {
            opcode: 0xEA,
            name: "SET 5 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xEB, Instruction {
            opcode: 0xEB,
            name: "SET 5 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xEC, Instruction {
            opcode: 0xEC,
            name: "SET 5 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xED, Instruction {
            opcode: 0xED,
            name: "SET 5 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xEE, Instruction {
            opcode: 0xEE,
            name: "SET 5 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xEF, Instruction {
            opcode: 0xEF,
            name: "SET 5 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF0, Instruction {
            opcode: 0xF0,
            name: "SET 6 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF1, Instruction {
            opcode: 0xF1,
            name: "SET 6 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF2, Instruction {
            opcode: 0xF2,
            name: "SET 6 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF3, Instruction {
            opcode: 0xF3,
            name: "SET 6 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF4, Instruction {
            opcode: 0xF4,
            name: "SET 6 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF5, Instruction {
            opcode: 0xF5,
            name: "SET 6 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF6, Instruction {
            opcode: 0xF6,
            name: "SET 6 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF7, Instruction {
            opcode: 0xF7,
            name: "SET 6 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF8, Instruction {
            opcode: 0xF8,
            name: "SET 7 B",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF9, Instruction {
            opcode: 0xF9,
            name: "SET 7 C",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xFA, Instruction {
            opcode: 0xFA,
            name: "SET 7 D",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xFB, Instruction {
            opcode: 0xFB,
            name: "SET 7 E",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xFC, Instruction {
            opcode: 0xFC,
            name: "SET 7 H",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xFD, Instruction {
            opcode: 0xFD,
            name: "SET 7 L",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xFE, Instruction {
            opcode: 0xFE,
            name: "SET 7 (HL)",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xFF, Instruction {
            opcode: 0xFF,
            name: "SET 7 A",
            cycles: 2,
            size: 2,
            flags: &[],
        });
		
		all_instructions
    }

}