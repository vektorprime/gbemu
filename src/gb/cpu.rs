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
        let valid_cb_instructions =  Cpu::setup_inst();
        loop {
            if !self.halted {
                let mut opcode = self.fetch_next_inst(mem);
                //if CB, read another byte, else decode and execute
                let mut cb_opcode = false;
                if opcode == 0xCB {
                    cb_opcode = true;
                    opcode = self.fetch_next_inst(mem);
                }
                let inst = if cb_opcode {
                    valid_cb_instructions.get(&opcode).unwrap()
                } else {
                    valid_instructions.get(&opcode).unwrap()
                };
                self.execute_inst(inst, mem);
            }
        }
    }

    pub fn fetch_next_inst(&mut self, mem: &Ram) -> u8 {
        let pc_reg = self.registers.get_and_inc_pc();
        mem.read(pc_reg)
    }



    pub fn execute_inst(&mut self,  inst: &Instruction, mem: &mut Ram) {
        // todo
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
                let bc = self.registers.get_bc();
                let hl = self.registers.get_hl();
                let (new_val, overflowed) = hl.overflowing_add(bc);
                if overflowed {
                    self.registers.set_c_flag();
                }
                else {
                    self.registers.set_hl(new_val);
                }
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
                let hl = self.registers.get_hl();
                let de = self.registers.get_de();
                let (new_val, overflowed) = hl.overflowing_add(de);
                // check for 12 bit overflow and set h flag
                let overflowed_12bit_max = 4096;
                if new_val > overflowed_12bit_max {
                    self.registers.set_h_flag();
                }
                // check for 16 bit overflow and set c flag
                if overflowed {
                    self.registers.set_c_flag();
                }
                else {
                    self.registers.set_hl(new_val);
                }
                self.registers.handle_flags(inst.name, );
                self.inc_cycles_by_inst_val(inst.cycles);
                self.registers.inc_pc_by_inst_val(inst.size);
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
                // LD (HL), B
                let addr = self.registers.get_hl();
                mem.write(addr, self.registers.get_b());
                self.registers.handle_flags(inst.name);
                self.inc_cycles_by_inst_val(inst.cycles);
                self.registers.inc_pc_by_inst_val(inst.size);
            },
            0x71 => {
                // LD (HL), C
                let addr = self.registers.get_hl();
                mem.write(addr, self.registers.get_c());
                self.registers.handle_flags(inst.name);
                self.inc_cycles_by_inst_val(inst.cycles);
                self.registers.inc_pc_by_inst_val(inst.size);
            },
            0x72 => {
                // LD (HL), D
                let addr = self.registers.get_hl();
                mem.write(addr, self.registers.get_d());
                self.registers.handle_flags(inst.name);
                self.inc_cycles_by_inst_val(inst.cycles);
                self.registers.inc_pc_by_inst_val(inst.size);
            },
            0x73 => {
                // LD (HL), E
                let addr = self.registers.get_hl();
                mem.write(addr, self.registers.get_e());
                self.registers.handle_flags(inst.name);
                self.inc_cycles_by_inst_val(inst.cycles);
                self.registers.inc_pc_by_inst_val(inst.size);
            },
            0x74 => {
                // LD (HL), H
                let addr = self.registers.get_hl();
                mem.write(addr, self.registers.get_h());
                self.registers.handle_flags(inst.name);
                self.inc_cycles_by_inst_val(inst.cycles);
                self.registers.inc_pc_by_inst_val(inst.size);
            },
            0x75 => {
                // LD (HL), L
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
                // LD (HL), A
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

   
    pub fn setup_inst() -> HashMap<u8, Instruction> {
        // https://meganesu.github.io/generate-gb-opcodes/
        let mut all_instructions = HashMap::new();
        all_instructions.insert(0x00, Instruction {
            opcode: 0x00,
            name: "NOP",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x01, Instruction {
            opcode: 0x01,
            name: "LD_BC_D16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0x02, Instruction {
            opcode: 0x02,
            name: "LD_(BC)_A",
            cycles: 1,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x03, Instruction {
            opcode: 0x03,
            name: "INC_BC",
            cycles: 1,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x04, Instruction {
            opcode: 0x04,
            name: "INC_B",
            cycles: 1,
            size: 1,
            flags: &[FlagBits::H],
        });
        all_instructions.insert(0x05, Instruction {
            opcode: 0x05,
            name: "DEC_B",
            cycles: 1,
            size: 1,
            flags: &[FlagBits::Z, FlagBits::H],
        });
        all_instructions.insert(0x06, Instruction {
            opcode: 0x06,
            name: "LD_B_D8",
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
            name: "LD_(A16)_SP",
            cycles: 3,
            size: 5,
            flags: &[FlagBits::C],
        });
        all_instructions.insert(0x09, Instruction {
            opcode: 0x09,
            name: "ADD_HL_BC",
            cycles: 0,
            size: 1,
            flags: &[FlagBits::C],
        });
        all_instructions.insert(0x0A, Instruction {
            opcode: 0x0A,
            name: "LD_A_(BC)",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0B, Instruction {
            opcode: 0x0B,
            name: "UNIMPLEMENTED_0B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0C, Instruction {
            opcode: 0x0C,
            name: "UNIMPLEMENTED_0C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0D, Instruction {
            opcode: 0x0D,
            name: "UNIMPLEMENTED_0D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0E, Instruction {
            opcode: 0x0E,
            name: "UNIMPLEMENTED_0E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x0F, Instruction {
            opcode: 0x0F,
            name: "UNIMPLEMENTED_0F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x10, Instruction {
            opcode: 0x10,
            name: "UNIMPLEMENTED_10",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x11, Instruction {
            opcode: 0x11,
            name: "UNIMPLEMENTED_11",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x12, Instruction {
            opcode: 0x12,
            name: "UNIMPLEMENTED_12",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x13, Instruction {
            opcode: 0x13,
            name: "UNIMPLEMENTED_13",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x14, Instruction {
            opcode: 0x14,
            name: "UNIMPLEMENTED_14",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x15, Instruction {
            opcode: 0x15,
            name: "UNIMPLEMENTED_15",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x16, Instruction {
            opcode: 0x16,
            name: "UNIMPLEMENTED_16",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x17, Instruction {
            opcode: 0x17,
            name: "UNIMPLEMENTED_17",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x18, Instruction {
            opcode: 0x18,
            name: "UNIMPLEMENTED_18",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x19, Instruction {
            opcode: 0x19,
            name: "UNIMPLEMENTED_19",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1A, Instruction {
            opcode: 0x1A,
            name: "UNIMPLEMENTED_1A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1B, Instruction {
            opcode: 0x1B,
            name: "UNIMPLEMENTED_1B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1C, Instruction {
            opcode: 0x1C,
            name: "UNIMPLEMENTED_1C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1D, Instruction {
            opcode: 0x1D,
            name: "UNIMPLEMENTED_1D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1E, Instruction {
            opcode: 0x1E,
            name: "UNIMPLEMENTED_1E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x1F, Instruction {
            opcode: 0x1F,
            name: "UNIMPLEMENTED_1F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x20, Instruction {
            opcode: 0x20,
            name: "UNIMPLEMENTED_20",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x21, Instruction {
            opcode: 0x21,
            name: "UNIMPLEMENTED_21",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x22, Instruction {
            opcode: 0x22,
            name: "UNIMPLEMENTED_22",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x23, Instruction {
            opcode: 0x23,
            name: "UNIMPLEMENTED_23",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x24, Instruction {
            opcode: 0x24,
            name: "UNIMPLEMENTED_24",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x25, Instruction {
            opcode: 0x25,
            name: "UNIMPLEMENTED_25",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x26, Instruction {
            opcode: 0x26,
            name: "UNIMPLEMENTED_26",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x27, Instruction {
            opcode: 0x27,
            name: "UNIMPLEMENTED_27",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x28, Instruction {
            opcode: 0x28,
            name: "UNIMPLEMENTED_28",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x29, Instruction {
            opcode: 0x29,
            name: "UNIMPLEMENTED_29",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2A, Instruction {
            opcode: 0x2A,
            name: "UNIMPLEMENTED_2A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2B, Instruction {
            opcode: 0x2B,
            name: "UNIMPLEMENTED_2B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2C, Instruction {
            opcode: 0x2C,
            name: "UNIMPLEMENTED_2C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2D, Instruction {
            opcode: 0x2D,
            name: "UNIMPLEMENTED_2D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2E, Instruction {
            opcode: 0x2E,
            name: "UNIMPLEMENTED_2E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2F, Instruction {
            opcode: 0x2F,
            name: "UNIMPLEMENTED_2F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x30, Instruction {
            opcode: 0x30,
            name: "UNIMPLEMENTED_30",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x31, Instruction {
            opcode: 0x31,
            name: "UNIMPLEMENTED_31",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x32, Instruction {
            opcode: 0x32,
            name: "UNIMPLEMENTED_32",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x33, Instruction {
            opcode: 0x33,
            name: "UNIMPLEMENTED_33",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x34, Instruction {
            opcode: 0x34,
            name: "UNIMPLEMENTED_34",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x35, Instruction {
            opcode: 0x35,
            name: "UNIMPLEMENTED_35",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x36, Instruction {
            opcode: 0x36,
            name: "UNIMPLEMENTED_36",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x37, Instruction {
            opcode: 0x37,
            name: "UNIMPLEMENTED_37",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x38, Instruction {
            opcode: 0x38,
            name: "UNIMPLEMENTED_38",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x39, Instruction {
            opcode: 0x39,
            name: "UNIMPLEMENTED_39",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3A, Instruction {
            opcode: 0x3A,
            name: "UNIMPLEMENTED_3A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3B, Instruction {
            opcode: 0x3B,
            name: "UNIMPLEMENTED_3B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3C, Instruction {
            opcode: 0x3C,
            name: "UNIMPLEMENTED_3C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3D, Instruction {
            opcode: 0x3D,
            name: "UNIMPLEMENTED_3D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3E, Instruction {
            opcode: 0x3E,
            name: "UNIMPLEMENTED_3E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x3F, Instruction {
            opcode: 0x3F,
            name: "UNIMPLEMENTED_3F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x40, Instruction {
            opcode: 0x40,
            name: "UNIMPLEMENTED_40",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x41, Instruction {
            opcode: 0x41,
            name: "UNIMPLEMENTED_41",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x42, Instruction {
            opcode: 0x42,
            name: "UNIMPLEMENTED_42",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x43, Instruction {
            opcode: 0x43,
            name: "UNIMPLEMENTED_43",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x44, Instruction {
            opcode: 0x44,
            name: "UNIMPLEMENTED_44",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x45, Instruction {
            opcode: 0x45,
            name: "UNIMPLEMENTED_45",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x46, Instruction {
            opcode: 0x46,
            name: "UNIMPLEMENTED_46",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x47, Instruction {
            opcode: 0x47,
            name: "UNIMPLEMENTED_47",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x48, Instruction {
            opcode: 0x48,
            name: "UNIMPLEMENTED_48",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x49, Instruction {
            opcode: 0x49,
            name: "UNIMPLEMENTED_49",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4A, Instruction {
            opcode: 0x4A,
            name: "UNIMPLEMENTED_4A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4B, Instruction {
            opcode: 0x4B,
            name: "UNIMPLEMENTED_4B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4C, Instruction {
            opcode: 0x4C,
            name: "UNIMPLEMENTED_4C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4D, Instruction {
            opcode: 0x4D,
            name: "UNIMPLEMENTED_4D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4E, Instruction {
            opcode: 0x4E,
            name: "UNIMPLEMENTED_4E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x4F, Instruction {
            opcode: 0x4F,
            name: "UNIMPLEMENTED_4F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x50, Instruction {
            opcode: 0x50,
            name: "UNIMPLEMENTED_50",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x51, Instruction {
            opcode: 0x51,
            name: "UNIMPLEMENTED_51",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x52, Instruction {
            opcode: 0x52,
            name: "UNIMPLEMENTED_52",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x53, Instruction {
            opcode: 0x53,
            name: "UNIMPLEMENTED_53",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x54, Instruction {
            opcode: 0x54,
            name: "UNIMPLEMENTED_54",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x55, Instruction {
            opcode: 0x55,
            name: "UNIMPLEMENTED_55",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x56, Instruction {
            opcode: 0x56,
            name: "UNIMPLEMENTED_56",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x57, Instruction {
            opcode: 0x57,
            name: "UNIMPLEMENTED_57",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x58, Instruction {
            opcode: 0x58,
            name: "UNIMPLEMENTED_58",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x59, Instruction {
            opcode: 0x59,
            name: "UNIMPLEMENTED_59",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5A, Instruction {
            opcode: 0x5A,
            name: "UNIMPLEMENTED_5A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5B, Instruction {
            opcode: 0x5B,
            name: "UNIMPLEMENTED_5B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5C, Instruction {
            opcode: 0x5C,
            name: "UNIMPLEMENTED_5C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5D, Instruction {
            opcode: 0x5D,
            name: "UNIMPLEMENTED_5D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5E, Instruction {
            opcode: 0x5E,
            name: "UNIMPLEMENTED_5E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x5F, Instruction {
            opcode: 0x5F,
            name: "UNIMPLEMENTED_5F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x60, Instruction {
            opcode: 0x60,
            name: "UNIMPLEMENTED_60",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x61, Instruction {
            opcode: 0x61,
            name: "UNIMPLEMENTED_61",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x62, Instruction {
            opcode: 0x62,
            name: "UNIMPLEMENTED_62",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x63, Instruction {
            opcode: 0x63,
            name: "UNIMPLEMENTED_63",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x64, Instruction {
            opcode: 0x64,
            name: "UNIMPLEMENTED_64",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x65, Instruction {
            opcode: 0x65,
            name: "UNIMPLEMENTED_65",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x66, Instruction {
            opcode: 0x66,
            name: "UNIMPLEMENTED_66",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x67, Instruction {
            opcode: 0x67,
            name: "UNIMPLEMENTED_67",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x68, Instruction {
            opcode: 0x68,
            name: "UNIMPLEMENTED_68",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x69, Instruction {
            opcode: 0x69,
            name: "UNIMPLEMENTED_69",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6A, Instruction {
            opcode: 0x6A,
            name: "UNIMPLEMENTED_6A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6B, Instruction {
            opcode: 0x6B,
            name: "UNIMPLEMENTED_6B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6C, Instruction {
            opcode: 0x6C,
            name: "UNIMPLEMENTED_6C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6D, Instruction {
            opcode: 0x6D,
            name: "UNIMPLEMENTED_6D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6E, Instruction {
            opcode: 0x6E,
            name: "UNIMPLEMENTED_6E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x6F, Instruction {
            opcode: 0x6F,
            name: "UNIMPLEMENTED_6F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x70, Instruction {
            opcode: 0x70,
            name: "UNIMPLEMENTED_70",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x71, Instruction {
            opcode: 0x71,
            name: "UNIMPLEMENTED_71",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x72, Instruction {
            opcode: 0x72,
            name: "UNIMPLEMENTED_72",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x73, Instruction {
            opcode: 0x73,
            name: "UNIMPLEMENTED_73",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x74, Instruction {
            opcode: 0x74,
            name: "UNIMPLEMENTED_74",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x75, Instruction {
            opcode: 0x75,
            name: "UNIMPLEMENTED_75",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x76, Instruction {
            opcode: 0x76,
            name: "UNIMPLEMENTED_76",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x77, Instruction {
            opcode: 0x77,
            name: "UNIMPLEMENTED_77",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x78, Instruction {
            opcode: 0x78,
            name: "UNIMPLEMENTED_78",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x79, Instruction {
            opcode: 0x79,
            name: "UNIMPLEMENTED_79",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7A, Instruction {
            opcode: 0x7A,
            name: "UNIMPLEMENTED_7A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7B, Instruction {
            opcode: 0x7B,
            name: "UNIMPLEMENTED_7B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7C, Instruction {
            opcode: 0x7C,
            name: "UNIMPLEMENTED_7C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7D, Instruction {
            opcode: 0x7D,
            name: "UNIMPLEMENTED_7D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7E, Instruction {
            opcode: 0x7E,
            name: "UNIMPLEMENTED_7E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x7F, Instruction {
            opcode: 0x7F,
            name: "UNIMPLEMENTED_7F",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x80, Instruction {
            opcode: 0x80,
            name: "UNIMPLEMENTED_80",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x81, Instruction {
            opcode: 0x81,
            name: "UNIMPLEMENTED_81",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x82, Instruction {
            opcode: 0x82,
            name: "UNIMPLEMENTED_82",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x83, Instruction {
            opcode: 0x83,
            name: "UNIMPLEMENTED_83",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x84, Instruction {
            opcode: 0x84,
            name: "UNIMPLEMENTED_84",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x85, Instruction {
            opcode: 0x85,
            name: "UNIMPLEMENTED_85",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x86, Instruction {
            opcode: 0x86,
            name: "UNIMPLEMENTED_86",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x87, Instruction {
            opcode: 0x87,
            name: "UNIMPLEMENTED_87",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x88, Instruction {
            opcode: 0x88,
            name: "UNIMPLEMENTED_88",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x89, Instruction {
            opcode: 0x89,
            name: "UNIMPLEMENTED_89",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8A, Instruction {
            opcode: 0x8A,
            name: "UNIMPLEMENTED_8A",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8B, Instruction {
            opcode: 0x8B,
            name: "UNIMPLEMENTED_8B",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8C, Instruction {
            opcode: 0x8C,
            name: "UNIMPLEMENTED_8C",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8D, Instruction {
            opcode: 0x8D,
            name: "UNIMPLEMENTED_8D",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8E, Instruction {
            opcode: 0x8E,
            name: "UNIMPLEMENTED_8E",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x8F, Instruction {
            opcode: 0x8F,
            name: "UNIMPLEMENTED_8F",
            cycles: 0,
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

    fn setup_inst_OLD() -> HashMap<u8, Instruction> {
        
        let mut all_instructions = HashMap::new();

        let inst_c3 = Instruction {
            // Load the 16-bit immediate operand a16 into the program counter (PC). a16 specifies the address of the subsequently executed instruction.
            // The second byte of the object code (immediately following the opcode) corresponds to the lower-order byte of a16 (bits 0-7), and the third byte of the object code corresponds to the higher-order byte (bits 8-15).
            opcode: 0xC3,
            name: "JP_A16",
            cycles: 4,
            size: 3,
            flags: &[],
        };

        all_instructions.insert(0xC3, inst_c3);

        let inst_fe = Instruction {
            // Compare the contents of register A and the contents of the 8-bit immediate operand d8 by calculating A - d8, and set the Z flag if they are equal.
            // The execution of this instruction does not affect the contents of register A.
            opcode: 0xFE,
            name: "CP",
            cycles: 4,
            size: 2,
            flags: &[FlagBits::Z],
        };

        all_instructions.insert(0xFE, inst_fe);

        let inst_20 = Instruction {
            // If the Z flag is 0, jump s8 steps from the current address stored in the program counter (PC). If not, the instruction following the current JP instruction is executed (as usual).
            opcode: 0x20,
            name: "JR NZ",
            cycles: 3,
            size: 2,
            flags: &[],
        };

        all_instructions.insert(0x20, inst_20);

        let inst_cd = Instruction {
            // In memory, push the program counter PC value corresponding to the address following the CALL instruction to the 2 bytes following the byte specified by the current stack pointer SP. Then load the 16-bit immediate operand a16 into PC.
            opcode: 0xCD,
            name: "CALL",
            cycles: 6,
            size: 3,
            flags: &[],
        };

        all_instructions.insert(0xCD, inst_cd);
        
        let inst_21 = Instruction {
            // Load the 2 bytes of immediate data into register pair HL.
            // The first byte of immediate data is the lower byte (i.e., bits 0-7), and the second byte of immediate data is the higher byte (i.e., bits 8-15).
            opcode: 0x21,
            name: "LD HL",
            cycles: 3,
            size: 3,
            flags: &[],
        };

        all_instructions.insert(0x21, inst_21);

        let inst_06 = Instruction {
            // Load the 8-bit immediate operand d8 into register B.
            opcode: 0x06,
            name: "LD B",
            cycles: 2,
            size: 2,
            flags: &[],
        };

        all_instructions.insert(0x06, inst_06);

        all_instructions
    }
}