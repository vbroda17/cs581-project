use std::{collections::VecDeque, fs::File, io::{BufRead, BufReader, Read}};

const WINDOW_SIZE: usize = 4096;

struct Window {
    buffer: [u8; WINDOW_SIZE],
    read_position: usize,
    write_position: usize,
}

impl Window {
    fn new() -> Self {
        let window = [0u8; WINDOW_SIZE];
        let read_position = 0;
        let write_position = 0;

        Self { buffer: window, read_position, write_position }
    }

    fn fill_from_file(&mut self, fin: &mut File) -> Result<usize, std::io::Error> {
        let mut bytes_read = 0;
        if self.write_position > self.read_position {
            bytes_read += fin.read(&mut self.buffer[self.write_position..])?;
            bytes_read += fin.read(&mut self.buffer[0..self.read_position])?;
        }
        else {
            bytes_read += fin.read(&mut self.buffer[self.write_position..self.read_position])?;
        }
        self.write_position = (self.read_position + bytes_read) % WINDOW_SIZE;

        Ok(bytes_read)
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        if self.write_position > self.read_position {
            if WINDOW_SIZE - self.write_position <= bytes.len() {
                self.buffer[self.write_position..(self.write_position + bytes.len())].copy_from_slice(&bytes[..]);
            }
            else {
                self.buffer[self.write_position..].copy_from_slice(&bytes[..(WINDOW_SIZE - self.write_position)]);
                self.buffer[..(bytes.len() - (WINDOW_SIZE - self.write_position))].copy_from_slice(&bytes[(WINDOW_SIZE - self.write_position)..]);
            }
        }
        else {
            self.buffer[self.write_position..(self.write_position + bytes.len())].copy_from_slice(&bytes[..]);
        }
    }

    // fn fill_from_other(&mut self, other: &mut Self) -> usize {
    //     let mut bytes_copied = 0;

    //     if self.write_position > self.read_position {
    //         if other.write_position > other.read_position {
                
    //         }
    //     }
    //     else {
    //         bytes_read += fin.read(&mut self.buffer[self.write_position..self.read_position])?;
    //     }
    // }
}

pub fn lz77(fin: &mut File) -> Result<(), std::io::Error> {
    let mut input = Window::new();
    let mut window = Window::new();

    input.fill_from_file(fin)?;

    Ok(())
}