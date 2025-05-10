use std::fs;
use std::io::ErrorKind;
use std::io::Write;

pub struct Rom {
    pub data: Vec<u8>,
}

impl Rom {
    pub fn new(file: &str) -> Self {
        Rom {
            data : fs::read(file).unwrap(),
        }
    }
}