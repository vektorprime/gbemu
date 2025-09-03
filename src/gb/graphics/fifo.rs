use std::collections::VecDeque;
use crate::gb::graphics::pixel::GBPixel;

pub enum FifoOpError {
    FifoFull,
    Empty,
}
pub struct Fifo {
    pub data: VecDeque<GBPixel>,
    pub max_size: usize,
}

impl Fifo {
    pub fn new() -> Self {
        Fifo {
            data: VecDeque::new(),
            max_size: 8,
        }
    }

    pub fn push(&mut self, pixel: GBPixel) -> Result<(), FifoOpError> {
        if self.data.len() >= self.max_size {
            Err(FifoOpError::FifoFull)
        }
        else {
            self.data.push_back(pixel);
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<GBPixel, FifoOpError> {
        self.data.pop_front().ok_or(FifoOpError::Empty)
    }
}