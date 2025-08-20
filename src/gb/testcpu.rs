use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::fs;
use crate::gb::registers::Registers;

#[derive(Debug)]
pub enum FailedRegister {
    pc(u16),
    sp(u16),
    a(u8),
    b(u8),
    c(u8),
    d(u8),
    e(u8),
    f(u8),
    h(u8),
    l(u8),
}


pub fn setup_initial_registers(registers: &mut Registers, initial_test_state: &TestState) {
    // println!("setting up initial registers");
    registers.set_pc(initial_test_state.pc);
    // println!("setting pc to 0x{:x}", initial_test_state.pc);
    registers.set_sp(initial_test_state.sp);
    // println!("setting sp to 0x{:x}", initial_test_state.sp);
    registers.set_a(initial_test_state.a);
    // println!("setting a to 0x{:x}", initial_test_state.a);
    registers.set_b(initial_test_state.b);
    // println!("setting b to 0x{:x}", initial_test_state.b);
    registers.set_c(initial_test_state.c);
    // println!("setting c to 0x{:x}", initial_test_state.c);
    registers.set_d(initial_test_state.d);
    // println!("setting d to 0x{:x}", initial_test_state.d);
    registers.set_e(initial_test_state.e);
    // println!("setting e to 0x{:x}", initial_test_state.e);
    registers.set_f(initial_test_state.f);
    // println!("setting f to 0x{:x}", initial_test_state.f);
    registers.set_h(initial_test_state.h);
    // println!("setting h to 0x{:x}", initial_test_state.h);
    registers.set_l(initial_test_state.l);
    // println!("setting l to 0x{:x}", initial_test_state.l);
}

pub fn compare_registers(registers: &Registers, final_test_state: &TestState) -> Vec<FailedRegister> {
    let mut failed_registers: Vec<FailedRegister> = Vec::new();

    let a = registers.get_a();
    if a != final_test_state.a {
        failed_registers.push(FailedRegister::a(a));
    }

    let b = registers.get_b();
    if registers.get_b() != final_test_state.b {
        failed_registers.push(FailedRegister::b(b));
    }

    let c = registers.get_c();
    if registers.get_c() != final_test_state.c {
        failed_registers.push(FailedRegister::c(c));
    }

    let d = registers.get_d();
    if d != final_test_state.d {
        failed_registers.push(FailedRegister::d(d));
    }

    let e = registers.get_e();
    if e != final_test_state.e {
        failed_registers.push(FailedRegister::e(e));
    }

    let f = registers.get_f();
    if f != final_test_state.f {
        failed_registers.push(FailedRegister::f(f));
    }

    let h = registers.get_h();
    if h != final_test_state.h {
        failed_registers.push(FailedRegister::h(h));
    }

    let l = registers.get_l();
    if l != final_test_state.l {
        failed_registers.push(FailedRegister::l(l));
    }

    failed_registers
}

//  read test ram and write values to real ram
//  should be a vec and never bare

// read test registers and copy values to real registers
//  compare FinalTestRegisters with real registers
//  compare finalTestRam with real ram

pub struct TestOpCode {
    pub opcode: u8,
    pub param: u16,
    pub isCB: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestState {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    #[serde(rename = "ram")]
    pub test_ram: Vec<(u16, u8)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Test {
    //  name
    pub name: String,
    // initial TestState
    //  inital ram
    //  Vec of TestRam
    #[serde(rename = "initial")]
    pub initial_test_state: TestState,
    // final TestState
    // final registers
    // final vec of TestRam
    #[serde(rename = "final")]
    pub final_test_state: TestState,
}

// create tests and add to Vec

pub fn read_test_file(path: &str, all_tests: &mut Vec<Test>)  {
    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();
    let reader = BufReader::with_capacity(480_000, file);

    let mut tests: Vec<Test> = serde_json::from_reader(reader).unwrap();
    all_tests.append(&mut tests);
}


pub fn get_all_files_in_directory(path: &str) -> Vec<String> {

    let mut files: Vec<String> = Vec::new();

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        //println!("{:?}", entry.path().to_str().unwrap().to_string());
                        files.push(entry.path().to_str().unwrap().to_string());
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    files
}

pub fn get_all_tests() ->  Vec<Test> {
    let mut all_tests: Vec<Test> = Vec::new();
    let all_files = get_all_files_in_directory("tests");
    for file in &all_files {
        read_test_file(file, &mut all_tests);
    }
    all_tests
}
