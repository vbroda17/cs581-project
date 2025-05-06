use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    ops::Index,
};

const WINDOW_SIZE: usize = (1 << 16) - 1;
const SENTINEL: u8 = 0xff;

struct RingBuffer<const CAPACITY: usize> {
    buffer: [u8; CAPACITY],
    at: usize,
    size: usize,
}

impl<const CAPACITY: usize> RingBuffer<CAPACITY> {
    fn new() -> Self {
        let buffer = [0u8; CAPACITY];
        let at = 0;
        let size = 0;

        Self { buffer, at, size }
    }

    fn write_position(&self) -> usize {
        (self.at + self.size) % CAPACITY
    }

    fn fill_from_file(&mut self, fin: &mut File) -> Result<usize, std::io::Error> {
        let write_position = self.write_position();
        let mut bytes_read = 0;

        if write_position >= self.at {
            bytes_read += fin.read(&mut self.buffer[write_position..])?;
            bytes_read += fin.read(&mut self.buffer[..self.at])?;
        } else {
            bytes_read += fin.read(&mut self.buffer[write_position..self.at])?;
        }
        self.size += bytes_read;

        Ok(bytes_read)
    }

    fn wrapping_write(&mut self, bytes: &[u8]) -> usize {
        let write_position = self.write_position();
        let to_write = std::cmp::min(CAPACITY, bytes.len());
        let to_ignore = std::cmp::max(to_write as i64 - (CAPACITY - self.size) as i64, 0) as usize;

        if write_position + to_write > CAPACITY {
            self.buffer[write_position..].copy_from_slice(&bytes[..(CAPACITY - write_position)]);
            self.buffer[..(to_write - (CAPACITY - write_position))]
                .copy_from_slice(&bytes[(CAPACITY - write_position)..to_write]);
        } else {
            self.buffer[write_position..(write_position + to_write)]
                .copy_from_slice(&bytes[..to_write]);
        }

        self.size += to_write - to_ignore;
        self.at = (self.at + to_ignore) % CAPACITY;
        to_ignore
    }

    fn peak(&mut self, bytes: &mut [u8], start: usize) -> usize {
        let to_peak = std::cmp::min(self.size - start % CAPACITY, bytes.len());
        let start = (self.at + start) % CAPACITY;

        if start + to_peak > CAPACITY {
            bytes[..(CAPACITY - start)].copy_from_slice(&self.buffer[start..]);
            bytes[(CAPACITY - start)..to_peak]
                .copy_from_slice(&self.buffer[..(to_peak - (CAPACITY - start))]);
        } else {
            bytes[..to_peak].copy_from_slice(&self.buffer[start..(start + to_peak)]);
        }

        to_peak
    }

    fn clear(&mut self) {
        self.size = 0;
        self.at = 0;
    }

    fn ignore(&mut self, bytes: usize) -> usize {
        if bytes > self.size {
            let ret = self.size;
            self.clear();
            ret
        } else {
            self.at = (self.at + bytes) % CAPACITY;
            self.size -= bytes;
            bytes
        }
    }

    fn wrapping_push(&mut self, v: u8) {
        self.buffer[self.write_position()] = v;

        if self.size >= CAPACITY {
            self.at += 1;
        } else {
            self.size += 1;
        }
    }


    fn len(&self) -> usize {
        self.size
    }


}

impl<const CAPACITY: usize> Index<usize> for RingBuffer<CAPACITY> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[(self.at + index) % CAPACITY]
    }
}

fn find_match<const CAPACITY: usize>(
    input: &mut RingBuffer<CAPACITY>,
    window_size: usize,
) -> Result<(usize, usize), std::io::Error> {
    let mut match_position = 0;
    let mut match_len = 0;

    let mut longest_match_len = 0;
    let mut longest_match_position = 0;
    let mut i = 0;

    while match_len + window_size < input.len() && match_len < 0xff00 {
        // println!("{}, {}", input[i] as char, input[window_size + match_len] as char);
        if input[i] == input[window_size + match_len] {
            if match_len == 0 {
                match_position = i;
            }

            match_len += 1;
            i += 1;
        } else {
            if match_len > longest_match_len {
                longest_match_position = match_position;
                longest_match_len = match_len;
            } else if match_len == 0 {
                i += 1;
            }

            match_len = 0;
        }

        if match_len == 0 && i >= window_size {
            break;
        }
    }
    if match_len > longest_match_len {
        longest_match_position = match_position;
        longest_match_len = match_len;
    }

    if longest_match_len == 0 {
        Ok((0, 0))
    } else {
        Ok((window_size - longest_match_position, longest_match_len))
    }
}

fn put_char(writer: &mut BufWriter<&mut File>, c: u8) -> Result<(), std::io::Error> {
    if c == SENTINEL {
        writer.write(&[SENTINEL, SENTINEL])?;
    } else {
        writer.write(&[c])?;
    }

    Ok(())
}

pub fn lz77_compress(fin: &mut File, fout: &mut File) -> Result<(), std::io::Error> {
    let mut input = RingBuffer::<{ 3 * WINDOW_SIZE }>::new();
    let mut window_size = 1;
    let mut writer = BufWriter::new(fout);

    input.fill_from_file(fin)?;
    put_char(&mut writer, input[0])?;
    while input.len() > window_size {
        let (d, l) = find_match(&mut input, window_size)?;
        let mut l = l;

        if l < 5 {
            if l == 0 {
                l += 1;
            }
            for i in 0..l {
                put_char(&mut writer, input[window_size + i])?;
            }
        } else {
            writer.write(&[SENTINEL])?;
            writer.write(&(l as u16).to_be_bytes())?;
            writer.write(&(d as u16).to_be_bytes())?;
        }

        let window_size_change = std::cmp::min(WINDOW_SIZE - window_size, l);
        window_size += window_size_change;
        input.ignore(l - window_size_change);

        if input.len() < 2 * WINDOW_SIZE {
            input.fill_from_file(fin)?;
        }
    }

    Ok(())
}

pub fn lz77_decompress(fin: &mut File, fout: &mut File) -> Result<(), std::io::Error> {
    let mut writer = BufWriter::new(fout);
    let mut window = RingBuffer::<WINDOW_SIZE>::new();
    let mut buf = [0u8; 0x2000];
    let mut buf2 = [0u8; 0x2000];
    let mut buf_size= fin.read(&mut buf)?;

    while buf_size > 0 {
        let mut i = 0;
        while i < buf_size && i < buf.len() - 4 {
            if buf[i] == SENTINEL {
                if buf[i + 1] == SENTINEL {
                    window.wrapping_push(buf[i]);
                    writer.write(&[buf[i]])?;
                    i += 2;
                } else {
                    let l = u16::from_be_bytes([buf[i + 1], buf[i + 2]]) as usize;
                    let d = u16::from_be_bytes([buf[i + 3], buf[i + 4]]) as usize;

                    let mut window_position = window.len() - d;
                    let mut bytes_copied = 0;
                    while bytes_copied < l {
                        let to_peak = std::cmp::min(l - bytes_copied, buf2.len());
                        let peaked = window.peak(&mut buf2[..to_peak], window_position);
                        let ignored = window.wrapping_write(&buf2[..peaked]);
                        window_position += peaked - ignored;
                        writer.write(&buf2[..peaked])?;
                        bytes_copied += peaked;
                    }

                    i += 5;
                }
            } else {
                writer.write(&[buf[i]])?;
                window.wrapping_push(buf[i]);
                i += 1;
            }
        }

        if i < buf_size {
            buf.copy_within(i..buf_size, 0);
            buf_size = fin.read(&mut buf[(buf_size - i)..])? + (buf_size - i);
        }
        else {
            buf_size = fin.read(&mut buf)?;
        }
    }

    Ok(())
}
