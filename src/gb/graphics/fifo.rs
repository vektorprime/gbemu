use std::collections::VecDeque;


pub enum FifoOpError {
    LenExceeded,
    Empty,
}
pub struct Fifo {
    pub data: [u8; 16],
    pub empty: bool,
    pub idx: usize,
}

impl Fifo {
    pub fn new() -> Self {
        Fifo {
            data: [0; 16],
            empty: true,
            idx: 0,
        }
    }
    pub fn clear(&mut self) {
        for i in 0..16 {
            self.data[i] = 0;
        }
        self.empty = true;
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn push(&mut self, data: u8) -> Result<(), FifoOpError> {
        if self.idx + 1 > self.data.len() {
            return Err(FifoOpError::LenExceeded);
        }
        self.data[self.idx] = data;
        self.idx += 1;
        if !self.empty {self.empty = false;}
        Ok(())
    }

    pub fn pop(&mut self) -> Result<u8, FifoOpError> {
        if self.idx == 0 {
            return Err(FifoOpError::Empty)
        }
        let popped_val = self.data[self.idx];
        self.idx -= 1;
        Ok(popped_val)
    }
}