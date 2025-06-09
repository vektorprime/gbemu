use crate::gb::registers::{Registers, InverseFlagBits, FlagBits, INVERSE_C_FLAG_BITS, INVERSE_H_FLAG_BITS, INVERSE_N_FLAG_BITS, INVERSE_Z_FLAG_BITS};
use crate::gb::hwregisters::*;
use crate::gb::instructions::Instruction; 
use crate::gb::mbc::*;
use crate::gb::bios::*;
use std::collections::HashMap;

pub const MAX_T_CYCLE_PER_FRAME: u64 = 70224;

pub struct Cpu {
    pub registers: Registers,
    pub ime: bool, // interrupt master
    pub pending_enable_ime: bool,
    pub pending_enable_ime_counter: u8,
    //pub opcode: u8, // opcode of current inst.
    pub cycles: u64, // total m cycle count
    pub last_cycles: u64,
    pub halted: bool,
    pub instructions: HashMap<u8, Instruction>,
    pub cb_instructions: HashMap<u8, Instruction>,
    pub bios_executed: bool,
    pub rom_loaded: bool,
}

impl Cpu { 
 
    pub fn new() -> Self {
        Cpu {
            registers: Registers::new(),
            ime: false,
            pending_enable_ime: false,
            pending_enable_ime_counter: 0,
            //opcode: 0,
            cycles: 0,
            last_cycles: 0,
            halted: false, 
            instructions: Cpu::setup_inst(),
            cb_instructions: Cpu::setup_cb_inst(),
            // set to true to test bios bypass
            bios_executed: true,
            rom_loaded: false,
        } 
    } 

    pub fn inc_cycles_by_inst_val(&mut self, size: u8) {
        
        self.cycles += size as u64;
        self.last_cycles = size as u64;
    }

    pub fn handle_interrupt(&mut self, mem: &mut Mbc) {
        if self.ime {
            // check that each interrupt is enabled and requested, then handle
            if mem.hw_reg.is_vblank_bit0_interrupt_enabled() {

            }
            if mem.hw_reg.is_lcd_stat_bit1_interrupt_enabled() {

            }
            if mem.hw_reg.is_timer_bit2_interrupt_enabled() {

            }
            if mem.hw_reg.is_serial_bit3_interrupt_enabled() {

            }
            if mem.hw_reg.is_joypad_bit4_interrupt_enabled() {

            }
        }
    }

    pub fn tick(&mut self, mem: &mut Mbc, bios: &Bios) -> u64 {
        let pc_print = self.registers.get_pc();
        println!("pc - 0x{:X}", pc_print);
        // debug
        if self.registers.get_pc() == 0x2820 {
            println!("pc is 0x2820");
        }
        // end debug
        if !self.bios_executed {
            // execute bios until end of data
            if self.registers.get_pc() < bios.data.len() as u16 {
                let mut opcode = self.fetch_next_inst(mem);
                //if CB, read another byte, else decode and execute
                let mut is_cb_opcode = false;
                if opcode == 0xCB {
                    is_cb_opcode = true;
                    opcode = self.fetch_next_inst(mem);
                } 
                let inst = if is_cb_opcode {
                    self.cb_instructions.get(&opcode).unwrap().clone()
                } else {
                    self.instructions.get(&opcode).unwrap().clone()
                };
                self.execute_inst(inst, mem, is_cb_opcode);
            }
            else {
                self.bios_executed = true;
                mem.load_rom_to_mem();
                self.rom_loaded = true;
            }
        }
        // else if !self.halted {
        else {
            // load rom here in case we want to test skipping bios
            if !self.rom_loaded {
                mem.load_rom_to_mem();
                self.rom_loaded = true;
            }
            let mut opcode = self.fetch_next_inst(mem);
            //if CB, read another byte, else decode and execute
            let mut is_cb_opcode = false;
            if opcode == 0xCB {
                is_cb_opcode = true;
                opcode = self.fetch_next_cb_inst(mem);
            }
            let inst = if is_cb_opcode {
                self.cb_instructions.get(&opcode).unwrap().clone()
            } else {
                self.instructions.get(&opcode).unwrap().clone()
            };
            self.execute_inst(inst, mem, is_cb_opcode);
        }

        if self.pending_enable_ime {
            self.pending_enable_ime_counter += 1;
            if self.pending_enable_ime_counter == 2 {
                self.ime = true;
                self.pending_enable_ime_counter = 0;
            }
        }
        self.last_cycles
    }

    pub fn fetch_next_inst(&mut self, mem: &Mbc) -> u8 {
        let pc_reg = self.registers.get_and_inc_pc();
        mem.read(pc_reg)
    }

    pub fn fetch_next_cb_inst(&mut self, mem: &Mbc) -> u8 {
        let pc_reg = self.registers.get_pc();
        mem.read(pc_reg)
    }

    pub fn execute_inst(&mut self,  inst: Instruction, mem: &mut Mbc, is_cb_opcode: bool) {
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
                    let pc = self.registers.get_pc();
                    let pc_offset_signed = mem.read(pc) as i8;
                    if pc_offset_signed < 0 {
                        let neg_offset = pc_offset_signed.abs() as u8;
                        // need to +1 because we start counting on the next op
                        let new_pc = pc - (neg_offset as u16) + 1;
                        self.registers.set_pc(new_pc);
                    }
                    else {
                        let offset = mem.read(pc) as u16;
                        // need to +1 because we start counting on the next op
                        let new_pc = pc + offset + 1;
                        self.registers.set_pc(new_pc);
                    }
                    //
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0x19 => {
                    // ADD HL DE
                    let hl = self.registers.get_hl();
                    let de = self.registers.get_de();

                    let (result, carry) = hl.overflowing_add(de);

                    // Half-carry: check if bit 11 overflowed
                    if ((hl & 0x0FFF) + (de & 0x0FFF)) > 0x0FFF {
                        self.registers.set_h_flag();
                    } else {
                        self.registers.clear_h_flag();
                    }

                    // Carry flag
                    if carry {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }

                    // always set val whether overflow or not
                    self.registers.set_hl(result);

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
                        let pc_offset_signed = mem.read(pc) as i8;
                        if pc_offset_signed < 0 {
                            let neg_offset = pc_offset_signed.abs() as u8;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc - (neg_offset as u16) + 1;
                            self.registers.set_pc(new_pc);
                        }
                        else {
                            let offset = mem.read(pc) as u16;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc + offset + 1;
                            self.registers.set_pc(new_pc);
                        }
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
                    // INC HL
                    self.registers.inc_hl();
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
                        let pc_offset_signed = mem.read(pc) as i8;
                        if pc_offset_signed < 0 {
                            let neg_offset = pc_offset_signed.abs() as u8;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc - (neg_offset as u16) + 1;
                            self.registers.set_pc(new_pc);
                        }
                        else {
                            let offset = mem.read(pc) as u16;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc + offset + 1;
                            self.registers.set_pc(new_pc);
                        }
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
                    if ((first_operand & 0x0FFF) + (second_operand & 0x0FFF)) > 0x0FFF {
                        self.registers.set_h_flag();
                    } else {
                        self.registers.clear_h_flag();
                    }
                    // check for 16 bit overflow and set c flag
                    if overflowed {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
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
                    if !c_flag {
                        let pc = self.registers.get_pc();
                        let pc_offset_signed = mem.read(pc) as i8;
                        if pc_offset_signed < 0 {
                            let neg_offset = pc_offset_signed.abs() as u8;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc - (neg_offset as u16) + 1;
                            self.registers.set_pc(new_pc);
                        }
                        else {
                            let offset = mem.read(pc) as u16;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc + offset + 1;
                            self.registers.set_pc(new_pc);
                        }
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
                        let pc_offset_signed = mem.read(pc) as i8;
                        if pc_offset_signed < 0 {
                            let neg_offset = pc_offset_signed.abs() as u8;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc - (neg_offset as u16) + 1;
                            self.registers.set_pc(new_pc);
                        }
                        else {
                            let offset = mem.read(pc) as u16;
                            // need to +1 because we start counting on the next op
                            let new_pc = pc + offset + 1;
                            self.registers.set_pc(new_pc);
                        }
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
                    if ((first_operand & 0x0FFF) + (second_operand & 0x0FFF)) > 0x0FFF {
                        self.registers.set_h_flag();
                    } else {
                        self.registers.clear_h_flag();
                    }
                    // check for 16 bit overflow and set c flag
                    if overflowed {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
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
                 0x98 => {
                    // SUBC A B
                    let a = self.registers.get_a();
                    let b = self.registers.get_b(); 
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                 0x99 => {
                    // SUBC A C
                    let a = self.registers.get_a();
                    let b = self.registers.get_c(); 
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9A => {
                    // SUBC A D
                    let a = self.registers.get_a();
                    let b = self.registers.get_d();
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },  
                0x9B => {
                    // SUBC A E
                    let a = self.registers.get_a();
                    let b = self.registers.get_e(); 
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },   
                0x9C => {
                    // SUBC A H
                    let a = self.registers.get_a();
                    let b = self.registers.get_h(); 
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },  
                0x9D => {
                    // SUBC A L
                    let a = self.registers.get_a();
                    let b = self.registers.get_l(); 
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                }, 
                0x9E => {
                    // SUBC A (HL)
                    let a = self.registers.get_a();
                    let addr = self.registers.get_hl(); 
                    let b = mem.read(addr);
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9F => {
                    // SUBC A A
                    let a = self.registers.get_a();
                    let b = self.registers.get_a(); 
                    let result = self.registers.sub_8bit_carry(a, b);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA0 => {
                    // AND B
                    let a = self.registers.get_a();
                    let b = self.registers.get_b(); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA1 => {
                    // AND B
                    let a = self.registers.get_a();
                    let b = self.registers.get_c(); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA2 => {
                    // AND D
                    let a = self.registers.get_a();
                    let b = self.registers.get_d(); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA3 => {
                    // AND E
                    let a = self.registers.get_a();
                    let b = self.registers.get_e(); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA4 => {
                    // AND H
                    let a = self.registers.get_a();
                    let b = self.registers.get_h(); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA5 => {
                    // AND L
                    let a = self.registers.get_a();
                    let b = self.registers.get_l(); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA6 => {
                    // AND (HL)
                    let a = self.registers.get_a();
                    let addr = self.registers.get_hl(); 
                    let b = mem.read(addr); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA7 => {
                    // AND A
                    let a = self.registers.get_a();
                    let b = self.registers.get_a(); 
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA8 => {
                    // XOR B
                    let a = self.registers.get_a();
                    let b = self.registers.get_b(); 
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA9 => {
                    // XOR B
                    let a = self.registers.get_a();
                    let b = self.registers.get_c(); 
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAA => {
                    // XOR D
                    let a = self.registers.get_a();
                    let b = self.registers.get_d(); 
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAB => {
                    // XOR E
                    let a = self.registers.get_a();
                    let b = self.registers.get_e();
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAC => {
                    // XOR H
                    let a = self.registers.get_a();
                    let b = self.registers.get_h();
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAD => {
                    // XOR L
                    let a = self.registers.get_a();
                    let b = self.registers.get_l();
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAE => {
                    // XOR (HL)
                    let a = self.registers.get_a();
                    let addr = self.registers.get_hl(); 
                    let b = mem.read(addr);
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAF => {
                    // XOR A
                    let a = self.registers.get_a();
                    let b = self.registers.get_a();
                    let result = a ^ b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB0 => {
                    // OR B
                    let a = self.registers.get_a();
                    let b = self.registers.get_b();
                    let result = a | b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB1 => {
                    // OR C
                    let a = self.registers.get_a();
                    let b = self.registers.get_c();
                    let result = a | b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB2 => {
                    // OR C
                    let a = self.registers.get_a();
                    let b = self.registers.get_d();
                    let result = a | b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB3 => {
                    // OR C
                    let a = self.registers.get_a();
                    let b = self.registers.get_e();
                    let result = a | b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB4 => {
                    // OR H
                    let a = self.registers.get_a();
                    let b = self.registers.get_h();
                    let result = a | b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB5 => {
                    // OR L
                    let a = self.registers.get_a();
                    let b = self.registers.get_l();
                    let result = a | b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB6 => {
                    // OR (HL)
                    let a = self.registers.get_a();
                    let addr = self.registers.get_hl();
                    let b = mem.read(addr);
                    let result = a | b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB7 => {
                    // OR A
                    let a = self.registers.get_a();
                    let result = a; 
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                 0xB8 => {
                    // CP B
                    let a = self.registers.get_a();
                    let b = self.registers.get_b();
                    let result = a - b;
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB9 => {
                    // CP C
                    let a = self.registers.get_a();
                    let b = self.registers.get_c();
                    let result = a.wrapping_sub(b);
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBA => {
                    // CP D
                    let a = self.registers.get_a();
                    let b = self.registers.get_d();
                    let result = a.wrapping_sub(b);
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBB => {
                    // CP E
                    let a = self.registers.get_a();
                    let b = self.registers.get_e();
                    let result = a.wrapping_sub(b);
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBC => {
                    // CP H
                    let a = self.registers.get_a();
                    let b = self.registers.get_h();
                    let result = a.wrapping_sub(b);
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBD => {
                    // CP L
                    let a = self.registers.get_a();
                    let b = self.registers.get_l();
                    let result = a.wrapping_sub(b);
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBE => {
                    // CP (HL)
                    let a = self.registers.get_a();
                    let addr = self.registers.get_hl();
                    let b = mem.read(addr);
                    let result = a.wrapping_sub(b);
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBF => {
                    // CP A
                    let a = self.registers.get_a();
                    let result = a.wrapping_sub(a);
                    if result == 0 {
                        self.registers.set_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC0 => {
                    // RET NZ
                    if !self.registers.is_z_flag_set() {
                        let address = self.registers.get_sp();
                        self.registers.set_pc(address);
                        self.registers.set_sp(address + 2);
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                },
                0xC1 => {
                    // POP BC
                    let address = self.registers.get_sp();
                    let lo = mem.read(address);
                    let hi = mem.read(address + 1);
                    self.registers.set_bc_with_two_val(lo, hi);
                    self.registers.set_sp(address + 2);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles); 
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC2 => {
                    // JP NZ A16
                    if !self.registers.is_z_flag_set() {
                        let lo = mem.read(self.registers.get_pc());
                        let hi = mem.read(self.registers.get_pc() + 1);
                        self.registers.set_pc(u16::from_le_bytes([lo, hi]));
                    }
                    else {
                        self.registers.set_pc(self.registers.get_pc() + 2);
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xC3 => {
                    // JP A16
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    self.registers.set_pc(u16::from_le_bytes([lo, hi]));
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xC4 => {
                    // CALL NZ A16
                    // if z is 0, set pc to a16, and push addr pc + 2 to stack
                    let ret_addr = self.registers.get_pc() + 2;
                    if !self.registers.is_z_flag_set() {
                        // get the called address and set pc to it
                        let lo = mem.read(self.registers.get_pc());
                        let hi = mem.read(self.registers.get_pc() + 1);
                        let called_addr = u16::from_le_bytes([lo, hi]);
                        self.registers.set_pc(called_addr);

                        // push return address to stack
                        // grow stack down
                        let sp_addr = self.registers.get_sp() - 2;
                        self.registers.set_sp(sp_addr);
                        // split ret_addr into two
                        let ret_part_1 = (ret_addr & 0x00FF) as u8;
                        mem.write(sp_addr, ret_part_1);
                        let ret_part_2 = (ret_addr >> 8) as u8;
                        mem.write(sp_addr + 1, ret_part_2);
                    }
                    else {
                        self.registers.set_pc(self.registers.get_pc() + 2);
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xC5 => {
                    // PUSH BC
                    // get the contents of BC
                    let bc = self.registers.get_bc();
                    let lo_bc = (bc & 0x00FF) as u8;
                    let hi_bc = (bc >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of bc in sp
                    // store msb of bc in sp + 1
                    mem.write(new_sp, lo_bc);
                    mem.write(new_sp + 1, hi_bc);

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC6 => {
                    // ADD A D8
                    let a = self.registers.get_a();
                    let imm = mem.read(self.registers.get_pc());
                    let result = a.wrapping_add(imm);
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.registers.inc_pc_by_inst_val(inst.size);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xC7 => {
                    // RST 0
                    // push pc to stack
                    let pc = self.registers.get_pc();
                    // split 16 bits to 2x 8 bit
                    let lo_pc = (pc & 0x00FF) as u8;
                    let hi_pc = (pc >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of bc in sp
                    // store msb of bc in sp + 1
                    mem.write(new_sp, lo_pc);
                    mem.write(new_sp + 1, hi_pc);
                    // set pc to 0x00
                    self.registers.set_pc(0x00);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xC9 => {
                    // RET
                    // get 16 bit add from SP
                    let sp = self.registers.get_sp();
                    let lo_sp = mem.read(sp);
                    let hi_sp = mem.read(sp + 1);
                    let ret_addr = u16::from_le_bytes([lo_sp, hi_sp]);
                    //set pc to 16 bit add
                    self.registers.set_pc(ret_addr);
                    // shrink stack by 2
                    let new_sp = self.registers.get_sp() + 2;
                    self.registers.set_sp(new_sp);
                    // set pc to ret addr
                    self.registers.set_pc(ret_addr);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xCD => {
                    // CALL A16
                    let pc = self.registers.get_pc();
                    let lo_call = mem.read(pc);
                    let hi_call = mem.read(pc + 1);
                    let target_addr = u16::from_le_bytes([lo_call, hi_call]);

                    // Return address = PC after the operand (2 bytes)
                    let return_addr = pc + 2;
                    let lo_pc = (return_addr & 0x00FF) as u8;
                    let hi_pc = (return_addr >> 8) as u8;

                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    mem.write(new_sp, lo_pc);          // write low byte
                    mem.write(new_sp + 1, hi_pc);      // write high byte

                    self.registers.set_pc(target_addr); //

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xCF => {
                    // RST 1
                    // push pc to stack
                    let pc = self.registers.get_pc();
                    // split 16 bits to 2x 8 bit
                    let lo_pc = (pc & 0x00FF) as u8;
                    let hi_pc = (pc >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of bc in sp
                    // store msb of bc in sp + 1
                    mem.write(new_sp, lo_pc);
                    mem.write(new_sp + 1, hi_pc);
                    // set pc to 0x08
                    self.registers.set_pc(0x08);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xD0 => {
                    // RET NC
                    if !self.registers.is_c_flag_set() {
                        let address = self.registers.get_sp();
                        self.registers.set_pc(address);
                        self.registers.set_sp(address + 2);
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD1 => {
                    // POP DE
                    let address = self.registers.get_sp();
                    let lo = mem.read(address);
                    let hi = mem.read(address + 1);
                    self.registers.set_de_with_two_val(lo, hi);
                    self.registers.set_sp(address + 2);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD5 => {
                    // PUSH DE
                    // get the contents of DE
                    let de = self.registers.get_de();
                    let lo_de = (de & 0x00FF) as u8;
                    let hi_de = (de >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of de in sp
                    // store msb of de in sp + 1
                    mem.write(new_sp, lo_de);
                    mem.write(new_sp + 1, hi_de);

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xDF => {
                    // RST 3
                    // push pc to stack
                    let pc = self.registers.get_pc();
                    // split 16 bits to 2x 8 bit
                    let lo_pc = (pc & 0x00FF) as u8;
                    let hi_pc = (pc >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of bc in sp
                    // store msb of bc in sp + 1
                    mem.write(new_sp, lo_pc);
                    mem.write(new_sp + 1, hi_pc);
                    // set pc to 0x18
                    self.registers.set_pc(0x18);
                },
                0xE0 => {
                    // LD (A8) A
                    // get the value in reg A
                    let a = self.registers.get_a();
                    // calculate the address with 0xFF00 + PC
                    let offset = mem.read(self.registers.get_pc()) as u16;
                    // Add the offset to 0xFF00
                    let base = 0xFF00;
                    let addr = base + offset;
                    // store the val from a in the addr
                    mem.write(addr, a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE1 => {
                    // POP HL
                    let address = self.registers.get_sp();
                    let lo = mem.read(address);
                    let hi = mem.read(address + 1);
                    self.registers.set_hl_with_two_val(lo, hi);
                    self.registers.set_sp(address + 2);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE2 => {
                    // LD (C) A
                    // get the value in reg A and C
                    let a = self.registers.get_a();
                    let offset = self.registers.get_c() as u16;
                    // calculate the address with 0xFF00 + C
                    // Add the offset to 0xFF00
                    let base = 0xFF00;
                    let addr = base + offset;
                    // store the val from a in the addr
                    mem.write(addr, a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE5 => {
                    // PUSH HL
                    // get the contents of HL
                    let hl = self.registers.get_hl();
                    let lo_hl = (hl & 0x00FF) as u8;
                    let hi_hl = (hl >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of hl in sp
                    // store msb of hl in sp + 1
                    mem.write(new_sp, lo_hl);
                    mem.write(new_sp + 1, hi_hl);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE6 => {
                    //AND D8
                    let a = self.registers.get_a();
                    let pc = self.registers.get_pc();
                    let b = mem.read(pc);
                    let result = a & b;
                    self.registers.set_a(result);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE9 => {
                    //JP HL
                    let hl = self.registers.get_hl();
                    self.registers.set_pc(hl);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                },
                0xEA => {
                    // LD (A16) A
                    // get the value in reg A
                    let a = self.registers.get_a();
                    // store it in the addr in next 2 bytes
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    let addr = u16::from_le_bytes([lo, hi]);
                    // store the val from a in the addr
                    mem.write(addr, a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xEF => {
                    // RST 5
                    // push pc to stack
                    let pc = self.registers.get_pc();
                    // split 16 bits to 2x 8 bit
                    let lo_pc = (pc & 0x00FF) as u8;
                    let hi_pc = (pc >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of bc in sp
                    // store msb of bc in sp + 1
                    mem.write(new_sp, lo_pc);
                    mem.write(new_sp + 1, hi_pc);
                    // set pc to 0x28
                    self.registers.set_pc(0x28);
                },
                0xF0 => {
                    // LD A (A8)
                    // get the value at PC
                    let offset = mem.read(self.registers.get_pc()) as u16;
                    // Add the offset to 0xFF00
                    let base = 0xFF00;
                    let addr = base + offset;
                    // deref that mem address
                    let val = mem.read(addr);
                    // store the value in A
                    self.registers.set_a(val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF2 => {
                    // LD A, (C)
                    // get the value of register C
                    let offset = self.registers.get_c() as u16;
                    // Add the offset to 0xFF00
                    let base = 0xFF00;
                    let addr = base + offset;
                    // deref that mem address
                    let val = mem.read(addr);
                    // store the value in A
                    self.registers.set_a(val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF3 => {
                    // DI
                    // set IME to enable immediately
                    self.ime = true;

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF5 => {
                    // PUSH AF
                    // get the contents of AF
                    let af = self.registers.get_af();
                    let lo_af = (af & 0x00FF) as u8;
                    let hi_af = (af >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp() - 2;
                    self.registers.set_sp(new_sp);
                    // store lsb of af in sp
                    // store msb of af in sp + 1
                    mem.write(new_sp, lo_af);
                    mem.write(new_sp + 1, hi_af);

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFA => {
                    // LD A, (a16)
                    // get the 16-bit immediate address from PC and PC+1
                    let lo = mem.read(self.registers.get_pc());
                    let hi = mem.read(self.registers.get_pc() + 1);
                    let addr = u16::from_le_bytes([lo, hi]);
                    // deref that mem address
                    let val = mem.read(addr);
                    // store the value in A
                    self.registers.set_a(val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFB => {
                    // IE
                    // set IME to enable AFTER the next inst. executes
                    self.pending_enable_ime = true;

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFE => {
                    // CP D8
                    let a = self.registers.get_a();
                    let val = mem.read(self.registers.get_pc());
                    if a.wrapping_sub(val) == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFF => {
                    // RST 7
                    // push pc to stack
                    let pc = self.registers.get_pc();
                    // split 16 bits to 2x 8 bit
                    let lo_pc = (pc & 0x00FF) as u8;
                    let hi_pc = (pc >> 8) as u8;
                    // grow stack down by 2
                    let new_sp = self.registers.get_sp().wrapping_sub(2);
                    self.registers.set_sp(new_sp);
                    // store lsb of bc in sp
                    // store msb of bc in sp + 1
                    mem.write(new_sp, lo_pc);
                    mem.write(new_sp + 1, hi_pc);
                    // set pc to 0x38
                    self.registers.set_pc(0x38);
                },
                _ => {
                    //todo
                    // panic here later once I've worked out the rest of code
                    let pc = self.registers.get_pc();
                    println!("current pc is 0x{:X}, unsure of opcode 0x{:X}", pc, inst.opcode);
                }
            } // match
        }
        else { // cb opcodes
            match inst.opcode {
                0x00 => {
                    // RLC B
                    //rotate register B left
                    //bit 7 is rotated to both C and bit 0 of reg B
                    let mut b = self.registers.get_b();
                    let select_bit7: u8 = 0b1000_0000;
                    let bit7 = b & select_bit7;
                    b <<= 1;
                    if bit7 == select_bit7 {
                        let select_bit0: u8 = 0b0000_0001;
                        b |= select_bit0;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if b == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x01 => {
                    // RLC C
                    let mut c = self.registers.get_c();
                    let bit7 = c & 0b1000_0000;
                    c <<= 1;
                    if bit7 != 0 {
                        c |= 0b0000_0001;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if c == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x02 => {
                    // RLC D
                    let mut d = self.registers.get_d();
                    let bit7 = d & 0b1000_0000;
                    d <<= 1;
                    if bit7 != 0 {
                        d |= 0b0000_0001;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if d == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x03 => {
                    // RLC E
                    let mut e = self.registers.get_e();
                    let bit7 = e & 0b1000_0000;
                    e <<= 1;
                    if bit7 != 0 {
                        e |= 0b0000_0001;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if e == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x04 => {
                    // RLC H
                    let mut h = self.registers.get_h();
                    let bit7 = h & 0b1000_0000;
                    h <<= 1;
                    if bit7 != 0 {
                        h |= 0b0000_0001;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if h == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x05 => {
                    // RLC L
                    let mut l = self.registers.get_l();
                    let bit7 = l & 0b1000_0000;
                    l <<= 1;
                    if bit7 != 0 {
                        l |= 0b0000_0001;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if l == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x06 => {
                    // RLC (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let bit7 = val & 0b1000_0000;
                    val <<= 1;
                    if bit7 != 0 {
                        val |= 0b0000_0001;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if val == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    mem.write(addr, val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x07 => {
                    // RLC A
                    let mut a = self.registers.get_a();
                    let bit7 = a & 0b1000_0000;
                    a <<= 1;
                    if bit7 != 0 {
                        a |= 0b0000_0001;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if a == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x08 => {
                    // RRC B
                    let mut b = self.registers.get_b();
                    let bit0 = b & 0b0000_0001;
                    b >>= 1;
                    if bit0 != 0 {
                        b |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if b == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x09 => {
                    // RRC C
                    let mut c = self.registers.get_c();
                    let bit0 = c & 0b0000_0001;
                    c >>= 1;
                    if bit0 != 0 {
                        c |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if c == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0A => {
                    // RRC D
                    let mut d = self.registers.get_d();
                    let bit0 = d & 0b0000_0001;
                    d >>= 1;
                    if bit0 != 0 {
                        d |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if d == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0B => {
                    // RRC E
                    let mut e = self.registers.get_e();
                    let bit0 = e & 0b0000_0001;
                    e >>= 1;
                    if bit0 != 0 {
                        e |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if e == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0C => {
                    // RRC H
                    let mut h = self.registers.get_h();
                    let bit0 = h & 0b0000_0001;
                    h >>= 1;
                    if bit0 != 0 {
                        h |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if h == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0D => {
                    // RRC L
                    let mut l = self.registers.get_l();
                    let bit0 = l & 0b0000_0001;
                    l >>= 1;
                    if bit0 != 0 {
                        l |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if l == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0E => {
                    // RRC (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let bit0 = val & 0b0000_0001;
                    val >>= 1;
                    if bit0 != 0 {
                        val |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if val == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    mem.write(addr, val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x0F => {
                    // RRC A
                    let mut a = self.registers.get_a();
                    let bit0 = a & 0b0000_0001;
                    a >>= 1;
                    if bit0 != 0 {
                        a |= 0b1000_0000;
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if a == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x10 => {
                    // RL B
                    let mut b = self.registers.get_b();
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = b & 0b1000_0000;
                    b = (b << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if b == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x11 => {
                    // RL C
                    let mut c = self.registers.get_c();
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = c & 0b1000_0000;
                    c = (c << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if c == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x12 => {
                    // RL D
                    let mut d = self.registers.get_d();
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = d & 0b1000_0000;
                    d = (d << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if d == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x13 => {
                    // RL E
                    let mut e = self.registers.get_e();
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = e & 0b1000_0000;
                    e = (e << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if e == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x14 => {
                    // RL H
                    let mut h = self.registers.get_h();
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = h & 0b1000_0000;
                    h = (h << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if h == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x15 => {
                    // RL L
                    let mut l = self.registers.get_l();
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = l & 0b1000_0000;
                    l = (l << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if l == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x16 => {
                    // RL (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = val & 0b1000_0000;
                    val = (val << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if val == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    mem.write(addr, val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x17 => {
                    // RL A
                    let mut a = self.registers.get_a();
                    let carry = if self.registers.is_c_flag_set() { 1 } else { 0 };
                    let bit7 = a & 0b1000_0000;
                    a = (a << 1) | carry;
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if a == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x18 => {
                    // RR B
                    let mut b = self.registers.get_b();
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = b & 0b0000_0001;
                    b >>= 1;
                    b |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if b == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x19 => {
                    // RR C
                    let mut c = self.registers.get_c();
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = c & 0b0000_0001;
                    c >>= 1;
                    c |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if c == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1A => {
                    // RR D
                    let mut d = self.registers.get_d();
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = d & 0b0000_0001;
                    d >>= 1;
                    d |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if d == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1B => {
                    // RR E
                    let mut e = self.registers.get_e();
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = e & 0b0000_0001;
                    e >>= 1;
                    e |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if e == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1C => {
                    // RR H
                    let mut h = self.registers.get_h();
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = h & 0b0000_0001;
                    h >>= 1;
                    h |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if h == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1D => {
                    // RR L
                    let mut l = self.registers.get_l();
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = l & 0b0000_0001;
                    l >>= 1;
                    l |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if l == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1E => {
                    // RR (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = val & 0b0000_0001;
                    val >>= 1;
                    val |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if val == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    mem.write(addr, val);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x1F => {
                    // RR A
                    let mut a = self.registers.get_a();
                    let carry = if self.registers.is_c_flag_set() { 0b1000_0000 } else { 0 };
                    let bit0 = a & 0b0000_0001;
                    a >>= 1;
                    a |= carry;
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if a == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x20 => {
                    // SLA B
                    let mut b = self.registers.get_b();
                    let bit7 = b & 0b1000_0000;
                    b <<= 1;
                    self.registers.set_b(b);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if b == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x21 => {
                    // SLA C
                    let mut c = self.registers.get_c();
                    let bit7 = c & 0b1000_0000;
                    c <<= 1;
                    self.registers.set_c(c);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if c == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x22 => {
                    // SLA D
                    let mut d = self.registers.get_d();
                    let bit7 = d & 0b1000_0000;
                    d <<= 1;
                    self.registers.set_d(d);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if d == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x23 => {
                    // SLA E
                    let mut e = self.registers.get_e();
                    let bit7 = e & 0b1000_0000;
                    e <<= 1;
                    self.registers.set_e(e);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if e == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x24 => {
                    // SLA H
                    let mut h = self.registers.get_h();
                    let bit7 = h & 0b1000_0000;
                    h <<= 1;
                    self.registers.set_h(h);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if h == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x25 => {
                    // SLA L
                    let mut l = self.registers.get_l();
                    let bit7 = l & 0b1000_0000;
                    l <<= 1;
                    self.registers.set_l(l);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if l == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x26 => {
                    // SLA (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let bit7 = val & 0b1000_0000;
                    val <<= 1;
                    mem.write(addr, val);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if val == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x27 => {
                    // SLA A
                    let mut a = self.registers.get_a();
                    let bit7 = a & 0b1000_0000;
                    a <<= 1;
                    self.registers.set_a(a);
                    if bit7 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if a == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x28 => {
                    // SRA B
                    let mut b = self.registers.get_b();
                    let bit0 = b & 0b0000_0001;
                    let msb = b & 0b1000_0000;
                    b >>= 1;
                    b |= msb;
                    self.registers.set_b(b);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if b == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x29 => {
                    // SRA C
                    let mut c = self.registers.get_c();
                    let bit0 = c & 0b0000_0001;
                    let msb = c & 0b1000_0000;
                    c >>= 1;
                    c |= msb;
                    self.registers.set_c(c);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if c == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2A => {
                    // SRA D
                    let mut d = self.registers.get_d();
                    let bit0 = d & 0b0000_0001;
                    let msb = d & 0b1000_0000;
                    d >>= 1;
                    d |= msb;
                    self.registers.set_d(d);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if d == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2B => {
                    // SRA E
                    let mut e = self.registers.get_e();
                    let bit0 = e & 0b0000_0001;
                    let msb = e & 0b1000_0000;
                    e >>= 1;
                    e |= msb;
                    self.registers.set_e(e);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if e == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2C => {
                    // SRA H
                    let mut h = self.registers.get_h();
                    let bit0 = h & 0b0000_0001;
                    let msb = h & 0b1000_0000;
                    h >>= 1;
                    h |= msb;
                    self.registers.set_h(h);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if h == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2D => {
                    // SRA L
                    let mut l = self.registers.get_l();
                    let bit0 = l & 0b0000_0001;
                    let msb = l & 0b1000_0000;
                    l >>= 1;
                    l |= msb;
                    self.registers.set_l(l);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if l == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2E => {
                    // SRA (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let bit0 = val & 0b0000_0001;
                    let msb = val & 0b1000_0000;
                    val >>= 1;
                    val |= msb;
                    mem.write(addr, val);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if val == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x2F => {
                    // SRA A
                    let mut a = self.registers.get_a();
                    let bit0 = a & 0b0000_0001;
                    let msb = a & 0b1000_0000;
                    a >>= 1;
                    a |= msb;
                    self.registers.set_a(a);
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    }
                    else {
                        self.registers.clear_c_flag();
                    }
                    if a == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x30 => {
                    // SWAP B
                    // swap the lower-order (4 bits) with the higher-order (4 bits) of B
                    // get lower 4 bits
                    // get higher 4 bits
                    // shift lower left 4 times
                    // shift higher right 4 times
                    // combine to one
                    // set B
                    let mut b = self.registers.get_b();
                    let select_lower_4bits: u8 = 0b0000_1111;
                    let select_higher_4bits: u8 = 0b1111_0000;
                    let mut lower_4bits = b & select_lower_4bits;
                    let mut higher_4bits = b & select_higher_4bits;
                    lower_4bits <<= 4;
                    higher_4bits >>= 4;
                    let mut new_b: u8 = 0;
                    new_b |= lower_4bits;
                    new_b |= higher_4bits;
                    self.registers.set_b(new_b);
                    if new_b == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }

                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x31 => {
                    // SWAP C
                    let mut c = self.registers.get_c();
                    let lower = c & 0b0000_1111;
                    let upper = c & 0b1111_0000;
                    let swapped = (lower << 4) | (upper >> 4);
                    self.registers.set_c(swapped);
                    self.registers.clear_c_flag();
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if swapped == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x32 => {
                    // SWAP D
                    let mut d = self.registers.get_d();
                    let lower = d & 0b0000_1111;
                    let upper = d & 0b1111_0000;
                    let swapped = (lower << 4) | (upper >> 4);
                    self.registers.set_d(swapped);
                    self.registers.clear_c_flag();
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if swapped == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x33 => {
                    // SWAP E
                    let mut e = self.registers.get_e();
                    let lower = e & 0b0000_1111;
                    let upper = e & 0b1111_0000;
                    let swapped = (lower << 4) | (upper >> 4);
                    self.registers.set_e(swapped);
                    self.registers.clear_c_flag();
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if swapped == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x34 => {
                    // SWAP H
                    let mut h = self.registers.get_h();
                    let lower = h & 0b0000_1111;
                    let upper = h & 0b1111_0000;
                    let swapped = (lower << 4) | (upper >> 4);
                    self.registers.set_h(swapped);
                    self.registers.clear_c_flag();
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if swapped == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x35 => {
                    // SWAP L
                    let mut l = self.registers.get_l();
                    let lower = l & 0b0000_1111;
                    let upper = l & 0b1111_0000;
                    let swapped = (lower << 4) | (upper >> 4);
                    self.registers.set_l(swapped);
                    self.registers.clear_c_flag();
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if swapped == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x36 => {
                    // SWAP (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let lower = val & 0b0000_1111;
                    let upper = val & 0b1111_0000;
                    let swapped = (lower << 4) | (upper >> 4);
                    mem.write(addr, swapped);
                    self.registers.clear_c_flag();
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if swapped == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x37 => {
                    // SWAP A
                    let mut a = self.registers.get_a();
                    let lower = a & 0b0000_1111;
                    let upper = a & 0b1111_0000;
                    let swapped = (lower << 4) | (upper >> 4);
                    self.registers.set_a(swapped);
                    self.registers.clear_c_flag();
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if swapped == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x38 => {
                    // SRL B
                    let mut b = self.registers.get_b();
                    let bit0 = b & 0b0000_0001;
                    b >>= 1;
                    self.registers.set_b(b);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if b == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x39 => {
                    // SRL C
                    let mut c = self.registers.get_c();
                    let bit0 = c & 0b0000_0001;
                    c >>= 1;
                    self.registers.set_c(c);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if c == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3A => {
                    // SRL D
                    let mut d = self.registers.get_d();
                    let bit0 = d & 0b0000_0001;
                    d >>= 1;
                    self.registers.set_d(d);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if d == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3B => {
                    // SRL E
                    let mut e = self.registers.get_e();
                    let bit0 = e & 0b0000_0001;
                    e >>= 1;
                    self.registers.set_e(e);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if e == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3C => {
                    // SRL H
                    let mut h = self.registers.get_h();
                    let bit0 = h & 0b0000_0001;
                    h >>= 1;
                    self.registers.set_h(h);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if h == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3D => {
                    // SRL L
                    let mut l = self.registers.get_l();
                    let bit0 = l & 0b0000_0001;
                    l >>= 1;
                    self.registers.set_l(l);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if l == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3E => {
                    // SRL (HL)
                    let addr = self.registers.get_hl();
                    let mut val = mem.read(addr);
                    let bit0 = val & 0b0000_0001;
                    val >>= 1;
                    mem.write(addr, val);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x3F => {
                    // SRL A
                    let mut a = self.registers.get_a();
                    let bit0 = a & 0b0000_0001;
                    a >>= 1;
                    self.registers.set_a(a);
                    self.registers.clear_h_flag();
                    self.registers.clear_n_flag();
                    if bit0 != 0 {
                        self.registers.set_c_flag();
                    } else {
                        self.registers.clear_c_flag();
                    }
                    if a == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x40 => {
                    // BIT 0 B
                    let b = self.registers.get_b();
                    let val = b & 0b0000_0001;
                    // complement of bit 0 is what we evaluate to set Z flag in BIT
                    if val == 0 {
                        self.registers.set_z_flag();
                    }
                    else {
                        self.registers.clear_z_flag();
                    }
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
                0x41 => {
                    // BIT 0 C
                    let c = self.registers.get_c();
                    let val = c & 0b0000_0001;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x42 => {
                    // BIT 0 D
                    let d = self.registers.get_d();
                    let val = d & 0b0000_0001;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x43 => {
                    // BIT 0 E
                    let e = self.registers.get_e();
                    let val = e & 0b0000_0001;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x44 => {
                    // BIT 0 H
                    let h = self.registers.get_h();
                    let val = h & 0b0000_0001;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x45 => {
                    // BIT 0 L
                    let l = self.registers.get_l();
                    let val = l & 0b0000_0001;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x46 => {
                    // BIT 0 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b0000_0001;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x47 => {
                    // BIT 0 A
                    let a = self.registers.get_a();
                    let val = a & 0b0000_0001;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x48 => {
                    // BIT 1 B
                    let b = self.registers.get_b();
                    let val = b & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x49 => {
                    // BIT 1 C
                    let c = self.registers.get_c();
                    let val = c & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4A => {
                    // BIT 1 D
                    let d = self.registers.get_d();
                    let val = d & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4B => {
                    // BIT 1 E
                    let e = self.registers.get_e();
                    let val = e & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4C => {
                    // BIT 1 H
                    let h = self.registers.get_h();
                    let val = h & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4D => {
                    // BIT 1 L
                    let l = self.registers.get_l();
                    let val = l & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4E => {
                    // BIT 1 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x4F => {
                    // BIT 1 A
                    let a = self.registers.get_a();
                    let val = a & 0b0000_0010;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x50 => {
                    // BIT 2 B
                    let b = self.registers.get_b();
                    let val = b & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x51 => {
                    // BIT 2 C
                    let c = self.registers.get_c();
                    let val = c & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x52 => {
                    // BIT 2 D
                    let d = self.registers.get_d();
                    let val = d & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x53 => {
                    // BIT 2 E
                    let e = self.registers.get_e();
                    let val = e & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x54 => {
                    // BIT 2 H
                    let h = self.registers.get_h();
                    let val = h & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x55 => {
                    // BIT 2 L
                    let l = self.registers.get_l();
                    let val = l & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x56 => {
                    // BIT 2 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x57 => {
                    // BIT 2 A
                    let a = self.registers.get_a();
                    let val = a & 0b0000_0100;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x58 => {
                    // BIT 3 B
                    let b = self.registers.get_b();
                    let val = b & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x59 => {
                    // BIT 3 C
                    let c = self.registers.get_c();
                    let val = c & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5A => {
                    // BIT 3 D
                    let d = self.registers.get_d();
                    let val = d & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5B => {
                    // BIT 3 E
                    let e = self.registers.get_e();
                    let val = e & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5C => {
                    // BIT 3 H
                    let h = self.registers.get_h();
                    let val = h & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5D => {
                    // BIT 3 L
                    let l = self.registers.get_l();
                    let val = l & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5E => {
                    // BIT 3 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x5F => {
                    // BIT 3 A
                    let a = self.registers.get_a();
                    let val = a & 0b0000_1000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x60 => {
                    // BIT 4 B
                    let b = self.registers.get_b();
                    let val = b & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x61 => {
                    // BIT 4 C
                    let c = self.registers.get_c();
                    let val = c & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x62 => {
                    // BIT 4 D
                    let d = self.registers.get_d();
                    let val = d & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x63 => {
                    // BIT 4 E
                    let e = self.registers.get_e();
                    let val = e & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x64 => {
                    // BIT 4 H
                    let h = self.registers.get_h();
                    let val = h & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x65 => {
                    // BIT 4 L
                    let l = self.registers.get_l();
                    let val = l & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x66 => {
                    // BIT 4 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x67 => {
                    // BIT 4 A
                    let a = self.registers.get_a();
                    let val = a & 0b0001_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x68 => {
                    // BIT 5 B
                    let b = self.registers.get_b();
                    let val = b & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x69 => {
                    // BIT 5 C
                    let c = self.registers.get_c();
                    let val = c & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6A => {
                    // BIT 5 D
                    let d = self.registers.get_d();
                    let val = d & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6B => {
                    // BIT 5 E
                    let e = self.registers.get_e();
                    let val = e & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6C => {
                    // BIT 5 H
                    let h = self.registers.get_h();
                    let val = h & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6D => {
                    // BIT 5 L
                    let l = self.registers.get_l();
                    let val = l & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6E => {
                    // BIT 5 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x6F => {
                    // BIT 5 A
                    let a = self.registers.get_a();
                    let val = a & 0b0010_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x70 => {
                    // BIT 6 B
                    let b = self.registers.get_b();
                    let val = b & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x71 => {
                    // BIT 6 C
                    let c = self.registers.get_c();
                    let val = c & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x72 => {
                    // BIT 6 D
                    let d = self.registers.get_d();
                    let val = d & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x73 => {
                    // BIT 6 E
                    let e = self.registers.get_e();
                    let val = e & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x74 => {
                    // BIT 6 H
                    let h = self.registers.get_h();
                    let val = h & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x75 => {
                    // BIT 6 L
                    let l = self.registers.get_l();
                    let val = l & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x76 => {
                    // BIT 6 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x77 => {
                    // BIT 6 A
                    let a = self.registers.get_a();
                    let val = a & 0b0100_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x78 => {
                    // BIT 7 B
                    let b = self.registers.get_b();
                    let val = b & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x79 => {
                    // BIT 7 C
                    let c = self.registers.get_c();
                    let val = c & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7A => {
                    // BIT 7 D
                    let d = self.registers.get_d();
                    let val = d & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7B => {
                    // BIT 7 E
                    let e = self.registers.get_e();
                    let val = e & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7C => {
                    // BIT 7 H
                    let h = self.registers.get_h();
                    let val = h & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7D => {
                    // BIT 7 L
                    let l = self.registers.get_l();
                    let val = l & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7E => {
                    // BIT 7 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr) & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x7F => {
                    // BIT 7 A
                    let a = self.registers.get_a();
                    let val = a & 0b1000_0000;
                    if val == 0 {
                        self.registers.set_z_flag();
                    } else {
                        self.registers.clear_z_flag();
                    }
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x80 => {
                    // RES 0 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b1111_1110;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x81 => {
                    // RES 0 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b1111_1110;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x82 => {
                    // RES 0 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b1111_1110;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x83 => {
                    // RES 0 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b1111_1110;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x84 => {
                    // RES 0 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b1111_1110;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x85 => {
                    // RES 0 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b1111_1110;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x86 => {
                    // RES 0 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b1111_1110);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x87 => {
                    // RES 0 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b1111_1110;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x88 => {
                    // RES 1 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b1111_1101;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x89 => {
                    // RES 1 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b1111_1101;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x8A => {
                    // RES 1 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b1111_1101;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x8B => {
                    // RES 1 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b1111_1101;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x8C => {
                    // RES 1 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b1111_1101;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x8D => {
                    // RES 1 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b1111_1101;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x8E => {
                    // RES 1 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b1111_1101);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x8F => {
                    // RES 1 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b1111_1101;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x90 => {
                    // RES 2 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b1111_1011;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x91 => {
                    // RES 2 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b1111_1011;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x92 => {
                    // RES 2 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b1111_1011;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x93 => {
                    // RES 2 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b1111_1011;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x94 => {
                    // RES 2 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b1111_1011;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x95 => {
                    // RES 2 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b1111_1011;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x96 => {
                    // RES 2 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b1111_1011);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x97 => {
                    // RES 2 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b1111_1011;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x98 => {
                    // RES 3 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b1111_0111;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x99 => {
                    // RES 3 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b1111_0111;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9A => {
                    // RES 3 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b1111_0111;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9B => {
                    // RES 3 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b1111_0111;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9C => {
                    // RES 3 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b1111_0111;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9D => {
                    // RES 3 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b1111_0111;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9E => {
                    // RES 3 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b1111_0111);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0x9F => {
                    // RES 3 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b1111_0111;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA0 => {
                    // RES 4 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b1110_1111;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA1 => {
                    // RES 4 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b1110_1111;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA2 => {
                    // RES 4 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b1110_1111;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA3 => {
                    // RES 4 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b1110_1111;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA4 => {
                    // RES 4 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b1110_1111;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA5 => {
                    // RES 4 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b1110_1111;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA6 => {
                    // RES 4 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b1110_1111);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA7 => {
                    // RES 4 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b1110_1111;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA8 => {
                    // RES 5 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b1101_1111;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xA9 => {
                    // RES 5 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b1101_1111;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAA => {
                    // RES 5 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b1101_1111;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAB => {
                    // RES 5 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b1101_1111;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAC => {
                    // RES 5 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b1101_1111;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAD => {
                    // RES 5 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b1101_1111;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAE => {
                    // RES 5 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b1101_1111);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xAF => {
                    // RES 5 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b1101_1111;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB0 => {
                    // RES 6 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b1011_1111;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB1 => {
                    // RES 6 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b1011_1111;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB2 => {
                    // RES 6 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b1011_1111;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB3 => {
                    // RES 6 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b1011_1111;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB4 => {
                    // RES 6 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b1011_1111;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB5 => {
                    // RES 6 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b1011_1111;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB6 => {
                    // RES 6 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b1011_1111);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB7 => {
                    // RES 6 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b1011_1111;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB8 => {
                    // RES 7 B
                    let mut b = self.registers.get_b();
                    let reset_bit: u8 = 0b0111_1111;
                    b &= reset_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xB9 => {
                    // RES 7 C
                    let mut c = self.registers.get_c();
                    let reset_bit: u8 = 0b0111_1111;
                    c &= reset_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBA => {
                    // RES 7 D
                    let mut d = self.registers.get_d();
                    let reset_bit: u8 = 0b0111_1111;
                    d &= reset_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBB => {
                    // RES 7 E
                    let mut e = self.registers.get_e();
                    let reset_bit: u8 = 0b0111_1111;
                    e &= reset_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBC => {
                    // RES 7 H
                    let mut h = self.registers.get_h();
                    let reset_bit: u8 = 0b0111_1111;
                    h &= reset_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBD => {
                    // RES 7 L
                    let mut l = self.registers.get_l();
                    let reset_bit: u8 = 0b0111_1111;
                    l &= reset_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBE => {
                    // RES 7 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val & 0b0111_1111);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xBF => {
                    // RES 7 A
                    let mut a = self.registers.get_a();
                    let reset_bit: u8 = 0b0111_1111;
                    a &= reset_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC0 => {
                    // SET 0 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b0000_0001;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC1 => {
                    // SET 0 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b0000_0001;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC2 => {
                    // SET 0 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b0000_0001;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC3 => {
                    // SET 0 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b0000_0001;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC4 => {
                    // SET 0 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b0000_0001;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC5 => {
                    // SET 0 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b0000_0001;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC6 => {
                    // SET 0 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b0000_0001);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC7 => {
                    // SET 0 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b0000_0001;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC8 => {
                    // SET 1 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b0000_0010;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xC9 => {
                    // SET 1 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b0000_0010;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xCA => {
                    // SET 1 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b0000_0010;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xCB => {
                    // SET 1 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b0000_0010;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xCC => {
                    // SET 1 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b0000_0010;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xCD => {
                    // SET 1 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b0000_0010;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xCE => {
                    // SET 1 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b0000_0010);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xCF => {
                    // SET 1 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b0000_0010;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD0 => {
                    // SET 2 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b0000_0100;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD1 => {
                    // SET 2 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b0000_0100;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD2 => {
                    // SET 2 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b0000_0100;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD3 => {
                    // SET 2 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b0000_0100;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD4 => {
                    // SET 2 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b0000_0100;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD5 => {
                    // SET 2 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b0000_0100;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD6 => {
                    // SET 2 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b0000_0100);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD7 => {
                    // SET 2 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b0000_0100;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD8 => {
                    // SET 3 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b0000_1000;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xD9 => {
                    // SET 3 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b0000_1000;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xDA => {
                    // SET 3 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b0000_1000;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xDB => {
                    // SET 3 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b0000_1000;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xDC => {
                    // SET 3 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b0000_1000;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xDD => {
                    // SET 3 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b0000_1000;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xDE => {
                    // SET 3 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b0000_1000);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xDF => {
                    // SET 3 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b0000_1000;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE0 => {
                    // SET 4 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b0001_0000;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE1 => {
                    // SET 4 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b0001_0000;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE2 => {
                    // SET 4 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b0001_0000;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE3 => {
                    // SET 4 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b0001_0000;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE4 => {
                    // SET 4 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b0001_0000;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE5 => {
                    // SET 4 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b0001_0000;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE6 => {
                    // SET 4 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b0001_0000);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE7 => {
                    // SET 4 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b0001_0000;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE8 => {
                    // SET 5 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b0010_0000;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xE9 => {
                    // SET 5 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b0010_0000;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xEA => {
                    // SET 5 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b0010_0000;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xEB => {
                    // SET 5 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b0010_0000;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xEC => {
                    // SET 5 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b0010_0000;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xED => {
                    // SET 5 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b0010_0000;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xEE => {
                    // SET 5 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b0010_0000);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xEF => {
                    // SET 5 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b0010_0000;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF0 => {
                    // SET 6 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b0100_0000;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF1 => {
                    // SET 6 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b0100_0000;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF2 => {
                    // SET 6 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b0100_0000;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF3 => {
                    // SET 6 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b0100_0000;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF4 => {
                    // SET 6 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b0100_0000;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF5 => {
                    // SET 6 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b0100_0000;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF6 => {
                    // SET 6 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b0100_0000);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF7 => {
                    // SET 6 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b0100_0000;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF8 => {
                    // SET 7 B
                    let mut b = self.registers.get_b();
                    let set_bit: u8 = 0b1000_0000;
                    b |= set_bit;
                    self.registers.set_b(b);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xF9 => {
                    // SET 7 C
                    let mut c = self.registers.get_c();
                    let set_bit: u8 = 0b1000_0000;
                    c |= set_bit;
                    self.registers.set_c(c);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFA => {
                    // SET 7 D
                    let mut d = self.registers.get_d();
                    let set_bit: u8 = 0b1000_0000;
                    d |= set_bit;
                    self.registers.set_d(d);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFB => {
                    // SET 7 E
                    let mut e = self.registers.get_e();
                    let set_bit: u8 = 0b1000_0000;
                    e |= set_bit;
                    self.registers.set_e(e);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFC => {
                    // SET 7 H
                    let mut h = self.registers.get_h();
                    let set_bit: u8 = 0b1000_0000;
                    h |= set_bit;
                    self.registers.set_h(h);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFD => {
                    // SET 7 L
                    let mut l = self.registers.get_l();
                    let set_bit: u8 = 0b1000_0000;
                    l |= set_bit;
                    self.registers.set_l(l);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFE => {
                    // SET 7 (HL)
                    let addr = self.registers.get_hl();
                    let val = mem.read(addr);
                    mem.write(addr, val | 0b1000_0000);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                0xFF => {
                    // SET 7 A
                    let mut a = self.registers.get_a();
                    let set_bit: u8 = 0b1000_0000;
                    a |= set_bit;
                    self.registers.set_a(a);
                    self.registers.handle_flags(inst.name);
                    self.inc_cycles_by_inst_val(inst.cycles);
                    self.registers.inc_pc_by_inst_val(inst.size);
                },
                _ => {
                    // todo
                    // panic here later once I've worked out the rest of code
                    let pc = self.registers.get_pc();
                    println!("current pc is 0x{:X}, unsure of opcode 0x{:X}", pc, inst.opcode);
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
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x28, Instruction {
            opcode: 0x28,
            name: "JR Z R8",
            cycles: 3, // or 8 if condition not met
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x29, Instruction {
            opcode: 0x29,
            name: "ADD HL HL",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2A, Instruction {
            opcode: 0x2A,
            name: "LD A (HL+)",
            cycles: 2,
            size: 1,
            flags: &[],
        });

        all_instructions.insert(0x2B, Instruction {
            opcode: 0x2B,
            name: "DEC HL",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2C, Instruction {
            opcode: 0x2C,
            name: "INC L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2D, Instruction {
            opcode: 0x2D,
            name: "DEC L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x2E, Instruction {
            opcode: 0x2E,
            name: "LD L D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x2F, Instruction {
            opcode: 0x2F,
            name: "CPL",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x30, Instruction {
            opcode: 0x30,
            name: "JR NC S8",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x31, Instruction {
            opcode: 0x31,
            name: "LD SP D16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0x32, Instruction {
            opcode: 0x32,
            name: "LD HLD A",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x33, Instruction {
            opcode: 0x33,
            name: "INC SP",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x34, Instruction {
            opcode: 0x34,
            name: "INC HL",
            cycles: 3,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x35, Instruction {
            opcode: 0x35,
            name: "DEC HL",
            cycles: 3,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x36, Instruction {
            opcode: 0x36,
            name: "LD HL D8",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x37, Instruction {
            opcode: 0x37,
            name: "SCF",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x38, Instruction {
            opcode: 0x38,
            name: "JR C R8",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0x39, Instruction {
            opcode: 0x39,
            name: "ADD HL SP",
            cycles: 2,
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
            name: "ADD A A",
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
            name: "SUB B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x91, Instruction {
            opcode: 0x91,
            name: "SUB C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x92, Instruction {
            opcode: 0x92,
            name: "SUB D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x93, Instruction {
            opcode: 0x93,
            name: "SUB E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x94, Instruction {
            opcode: 0x94,
            name: "SUB H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x95, Instruction {
            opcode: 0x95,
            name: "SUB L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x96, Instruction {
            opcode: 0x96,
            name: "SUB (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x97, Instruction {
            opcode: 0x97,
            name: "SUB A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x98, Instruction {
            opcode: 0x98,
            name: "SBC A, B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x99, Instruction {
            opcode: 0x99,
            name: "SBC A, C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9A, Instruction {
            opcode: 0x9A,
            name: "SBC A, D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9B, Instruction {
            opcode: 0x9B,
            name: "SBC A, E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9C, Instruction {
            opcode: 0x9C,
            name: "SBC A, H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9D, Instruction {
            opcode: 0x9D,
            name: "SBC A, L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9E, Instruction {
            opcode: 0x9E,
            name: "SBC A, (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0x9F, Instruction {
            opcode: 0x9F,
            name: "SBC A, A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA0, Instruction {
            opcode: 0xA0,
            name: "AND B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA1, Instruction {
            opcode: 0xA1,
            name: "AND C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA2, Instruction {
            opcode: 0xA2,
            name: "AND D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA3, Instruction {
            opcode: 0xA3,
            name: "AND E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA4, Instruction {
            opcode: 0xA4,
            name: "AND H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA5, Instruction {
            opcode: 0xA5,
            name: "AND L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA6, Instruction {
            opcode: 0xA6,
            name: "AND (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA7, Instruction {
            opcode: 0xA7,
            name: "AND A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA8, Instruction {
            opcode: 0xA8,
            name: "XOR B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xA9, Instruction {
            opcode: 0xA9,
            name: "XOR C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAA, Instruction {
            opcode: 0xAA,
            name: "XOR D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAB, Instruction {
            opcode: 0xAB,
            name: "XOR E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAC, Instruction {
            opcode: 0xAC,
            name: "XOR H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAD, Instruction {
            opcode: 0xAD,
            name: "XOR L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAE, Instruction {
            opcode: 0xAE,
            name: "XOR (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xAF, Instruction {
            opcode: 0xAF,
            name: "XOR A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB0, Instruction {
            opcode: 0xB0,
            name: "OR B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB1, Instruction {
            opcode: 0xB1,
            name: "OR C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB2, Instruction {
            opcode: 0xB2,
            name: "OR D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB3, Instruction {
            opcode: 0xB3,
            name: "OR E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB4, Instruction {
            opcode: 0xB4,
            name: "OR H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB5, Instruction {
            opcode: 0xB5,
            name: "OR L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB6, Instruction {
            opcode: 0xB6,
            name: "OR (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB7, Instruction {
            opcode: 0xB7,
            name: "OR A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB8, Instruction {
            opcode: 0xB8,
            name: "CP B",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xB9, Instruction {
            opcode: 0xB9,
            name: "CP C",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBA, Instruction {
            opcode: 0xBA,
            name: "CP D",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBB, Instruction {
            opcode: 0xBB,
            name: "CP E",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBC, Instruction {
            opcode: 0xBC,
            name: "CP H",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBD, Instruction {
            opcode: 0xBD,
            name: "CP L",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBE, Instruction {
            opcode: 0xBE,
            name: "CP (HL)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xBF, Instruction {
            opcode: 0xBF,
            name: "CP A",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC0, Instruction {
            opcode: 0xC0,
            name: "RET NZ",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC1, Instruction {
            opcode: 0xC1,
            name: "POP BC",
            cycles: 3,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC2, Instruction {
            opcode: 0xC2,
            name: "JP NZ A16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xC3, Instruction {
            opcode: 0xC3,
            name: "JP A16",
            cycles: 4,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xC4, Instruction {
            opcode: 0xC4,
            name: "CALL NZ, A16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xC5, Instruction {
            opcode: 0xC5,
            name: "PUSH BC",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC6, Instruction {
            opcode: 0xC6,
            name: "ADD A, D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xC7, Instruction {
            opcode: 0xC7,
            name: "RST 00H",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC8, Instruction {
            opcode: 0xC8,
            name: "RET Z",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xC9, Instruction {
            opcode: 0xC9,
            name: "RET",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCA, Instruction {
            opcode: 0xCA,
            name: "JP Z, A16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xCB, Instruction {
            opcode: 0xCB,
            name: "UNIMPLEMENTED_CB",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xCC, Instruction {
            opcode: 0xCC,
            name: "CALL Z, A16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xCD, Instruction {
            opcode: 0xCD,
            name: "CALL A16",
            cycles: 6,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xCE, Instruction {
            opcode: 0xCE,
            name: "ADC A, D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xCF, Instruction {
            opcode: 0xCF,
            name: "RST 08H",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD0, Instruction {
            opcode: 0xD0,
            name: "RET NC",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD1, Instruction {
            opcode: 0xD1,
            name: "POP DE",
            cycles: 3,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD2, Instruction {
            opcode: 0xD2,
            name: "JP NC, A16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xD3, Instruction {
            opcode: 0xD3,
            name: "UNUSED",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD4, Instruction {
            opcode: 0xD4,
            name: "CALL NC, A16",
            cycles: 3,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xD5, Instruction {
            opcode: 0xD5,
            name: "PUSH DE",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD6, Instruction {
            opcode: 0xD6,
            name: "SUB D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xD7, Instruction {
            opcode: 0xD7,
            name: "RST 10H",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD8, Instruction {
            opcode: 0xD8,
            name: "RET C",
            cycles: 5,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xD9, Instruction {
            opcode: 0xD9,
            name: "RETI",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDA, Instruction {
            opcode: 0xDA,
            name: "JP C A16",
            cycles: 4,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xDB, Instruction {
            opcode: 0xDB,
            name: "UNUSED",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDC, Instruction {
            opcode: 0xDC,
            name: "CALL C, A16",
            cycles: 6,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xDD, Instruction {
            opcode: 0xDD,
            name: "UNUSED",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xDE, Instruction {
            opcode: 0xDE,
            name: "SBC D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xDF, Instruction {
            opcode: 0xDF,
            name: "RST 18H",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE0, Instruction {
            opcode: 0xE0,
            name: "LD (A8) A",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE1, Instruction {
            opcode: 0xE1,
            name: "POP HL",
            cycles: 3,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE2, Instruction {
            opcode: 0xE2,
            name: "LD (C) A",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE3, Instruction {
            opcode: 0xE3,
            name: "UNIMPLEMENTED_E3",
            cycles: 2,
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
            name: "PUSH HL",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE6, Instruction {
            opcode: 0xE6,
            name: "AND D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE7, Instruction {
            opcode: 0xE7,
            name: "RST 4",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xE8, Instruction {
            opcode: 0xE8,
            name: "ADD SP S8",
            cycles: 4,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xE9, Instruction {
            opcode: 0xE9,
            name: "JP HL",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xEA, Instruction {
            opcode: 0xEA,
            name: "LD (A16) A",
            cycles: 4,
            size: 3,
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
            name: "XOR D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xEF, Instruction {
            opcode: 0xEF,
            name: "RST 5",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF0, Instruction {
            opcode: 0xF0,
            name: "LD A (A8)",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF1, Instruction {
            opcode: 0xF1,
            name: "POP AF",
            cycles: 3,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF2, Instruction {
            opcode: 0xF2,
            name: "LD A (C)",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF3, Instruction {
            opcode: 0xF3,
            name: "DI",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF4, Instruction {
            opcode: 0xF4,
            name: "UNUSED",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF5, Instruction {
            opcode: 0xF5,
            name: "PUSH AF",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF6, Instruction {
            opcode: 0xF6,
            name: "OR D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF7, Instruction {
            opcode: 0xF7,
            name: "RST 30H",
            cycles: 4,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xF8, Instruction {
            opcode: 0xF8,
            name: "LD HL SP+S8",
            cycles: 3,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xF9, Instruction {
            opcode: 0xF9,
            name: "LD SP HL",
            cycles: 2,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFA, Instruction {
            opcode: 0xFA,
            name: "LD A (A16)",
            cycles: 4,
            size: 3,
            flags: &[],
        });
        all_instructions.insert(0xFB, Instruction {
            opcode: 0xFB,
            name: "EI",
            cycles: 1,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFC, Instruction {
            opcode: 0xFC,
            name: "UNUSED",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFD, Instruction {
            opcode: 0xFD,
            name: "UNUSED",
            cycles: 0,
            size: 1,
            flags: &[],
        });
        all_instructions.insert(0xFE, Instruction {
            opcode: 0xFE,
            name: "CP D8",
            cycles: 2,
            size: 2,
            flags: &[],
        });
        all_instructions.insert(0xFF, Instruction {
            opcode: 0xFF,
            name: "RST 38H",
            cycles: 4,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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
            cycles: 1,
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