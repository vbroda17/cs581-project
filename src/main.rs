use std::fs::File;

use cs581_project::{huffman, lz77::{lz77_compress, lz77_decompress}};

const WINDOW_SIZE: usize = 1 << 12;

fn main() {
    let args :Vec<String> = std::env::args().collect();

    if args[1] == "compress" {
        lz77_compress(&mut File::open(args[2].as_str()).expect("Couldn't open input file."), &mut File::create(format!("{}.z", args[2])).expect("Couldn't create output file"), WINDOW_SIZE).expect("LZ77 file error");
        println!("lz77 complete");
        let ht = huffman::HuffmanTree::new(&File::open(format!("{}.z", args[2])).expect("Couldn't open file")).expect("Couldn't create huffman tree");
        println!("Huffman Tree complete");
        ht.encode(&File::open(format!("{}.z", args[2])).expect("Couldn't open file"), &File::create(format!("{}.hz", args[2])).expect("Couldn't  open huffman output")).expect("Couldn't perform huffman encoding");
        println!("Huffman Encoding Complete");

    }
    else if args[1] == "decompress" {
        lz77_decompress(&mut File::open(args[2].as_str()).expect("Couldn't open compressed file."), &mut File::create(args[2].trim_end_matches(".z")).expect("Couldn't create output file"), WINDOW_SIZE).expect("LZ77 file error");
    }


}
