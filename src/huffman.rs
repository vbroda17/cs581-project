use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::io::SeekFrom;
use std::mem::size_of;

struct Node {
    value: Option<u8>,
    frequency: u32,
    next: [Option<Box<Node>>; 2],
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.frequency.cmp(&other.frequency).reverse()
    }
}

pub struct HuffmanTree {
    root: Box<Node>,
    encodings: HashMap<u8, (u64, u8)>,
    pub frequencies: BTreeMap<u8, u32>
}

impl HuffmanTree{
    pub fn new(file: &File) -> Result<HuffmanTree, std::io::Error> {
        let reader = BufReader::new(file);
        let mut frequency_map = BTreeMap::new();
        let mut char_priority_queue = BinaryHeap::new();

        // Get frequencies
        for byte in reader.bytes() {
            let counter = frequency_map.entry(byte?).or_insert(0);
            *counter += 1;
        }

        for (byte, count) in &frequency_map {
            char_priority_queue.push(Box::new(Node {
                value: Some(*byte),
                frequency: *count,
                next: [None, None]
            }));
        }

        while char_priority_queue.len() > 1 {
            let left = char_priority_queue.pop();
            let right = char_priority_queue.pop();

            let frequency = match left.as_ref()  { None => 0, Some(value) => value.frequency } + 
                            match right.as_ref() { None => 0, Some(value) => value.frequency};


            let new_node = Box::new(Node {
                value: None,
                frequency: frequency,
                next: [left, right]
            });
            
            char_priority_queue.push(new_node);

        }

        let root = char_priority_queue.pop().unwrap();
        let mut stack = Vec::new();
        let mut encodings_map = HashMap::new();

        stack.push((&root, 0 as u64, 0u8));

        while stack.len() > 0 {
            let (node, encoding, depth) = stack.pop().unwrap();

            match node.value {
                None => (),
                Some(value) => {
                    encodings_map.insert(value, (encoding, depth));
                }
            }

            for i in 0..2 {
                match &node.next[i] {
                    None => (),
                    Some(link) => stack.push((link, encoding | ((i as u64) << depth), depth + 1 as u8))
                }
            }
        }

        // This won't panic since we know that the priority queue has length of 1
        Ok(HuffmanTree {root: root, encodings: encodings_map, frequencies: frequency_map}) 
    }

    pub fn print(&self) -> () {
        let mut traversal_queue = VecDeque::new();
        let mut current_depth = 1;

        traversal_queue.push_back((&self.root, 1));

        while !traversal_queue.is_empty() {
            let (node, depth) = match traversal_queue.pop_front() {
                None => panic!("This doesn't make any sense"),
                Some((node, depth)) => (node, depth),
            };

            if depth > current_depth {
                current_depth = depth;
                println!()
            }

            print!("({:?}:{:?})", node.frequency, match node.value {
                None => '\0',
                Some(value) => value as char,
            });

            for link in &node.next {
                match link {
                    None => (),
                    Some(link) => traversal_queue.push_back((link, current_depth + 1)),
                }
            }
        }
        println!()
    }

    pub fn encode(&self, message_file: &File, encoded_file: &File) -> Result<(), std::io::Error> {
        let message_r= BufReader::new(message_file);

        let mut encoded_w = BufWriter::new(encoded_file);
        let mut buf= 0u64;
        let mut pos = 0;
        let mut num_bits = 0usize;

        encoded_w.seek(SeekFrom::Start(size_of::<usize>() as u64))?;

        for byte in message_r.bytes() {
            let byte = byte?;
            match self.encodings.get(&byte) {
                None => panic!("Missing encoding for: {}",  byte),
                Some((encoding, length)) => {
                    buf |= encoding << pos;
                    pos += length;
                    num_bits += *length as usize;

                    if pos >= 64 {
                        pos -= 64;
                        encoded_w.write(&buf.to_le_bytes())?;
                        buf = encoding >> ((length - pos) as u64);
                    }
                }
            };
        }

        if pos > 0 {
            encoded_w.write(&buf.to_le_bytes())?;
        }

        encoded_w.rewind()?;
        encoded_w.write(&num_bits.to_le_bytes())?;

        Ok(())
    }

    pub fn decode(&self, encoded_file: &mut File, output_file: &File) -> Result<(), std::io::Error> {
        let mut writer = BufWriter::new(output_file);
        let mut usize_buffer = [0u8; size_of::<usize>()];
        let mut buf = [0u8; 4096];
        encoded_file.read(&mut usize_buffer)?;
        let num_bits = usize::from_le_bytes(usize_buffer);
        let mut current_node = &self.root;
        let mut buf_size;
        let mut bits_read = 0;


        buf_size = encoded_file.read(&mut buf)?;
        while buf_size > 0 {

            let bits_to_process = std::cmp::min(buf_size * 8, num_bits - bits_read);
            for i in 0..bits_to_process {
                let bit = ((buf[i / 8] >> (i % 8)) & 1) as usize;

                current_node = match &current_node.next[bit] {
                    None => panic!("Bad encoding"),
                    Some(node) => node
                };
    
                match &current_node.value {
                    None => (),
                    Some(value) => {
                        writer.write(&value.to_le_bytes())?;
                        current_node = &self.root;
                    }
                }
            }
            bits_read += bits_to_process;

            buf_size = encoded_file.read(&mut buf)?;
        }
        
        Ok(())
    }
}