use std::fs::{metadata, File};

use cs581_project::{huffman, lz77::{lz77_compress, lz77_decompress}};

fn compress_using_huffman(filepath: &String, output_extension: &str) -> String {
    let input_file = &File::open(filepath).expect("Couldn't open file");
    let ht = huffman::HuffmanTree::new(input_file).expect("Couldn't create huffman tree");

    let input_file = &File::open(filepath).expect("Couldn't open file");
    let output_filepath = format!("{}.{}", filepath, output_extension);
    let output_file = &File::create(&output_filepath).expect("Couldn't create output file");
    ht.encode(input_file, output_file).expect("Couldn't perform huffman encoding");

    return output_filepath;
}

fn compress_using_lz77(filepath: &String, window_size: usize) -> String {
    let input_file = &mut File::open(filepath).expect("Couldn't open file");
    let output_filepath = format!("{}.lz77", filepath);
    let output_file = &mut File::create(&output_filepath).expect("Couldn't create output file");
    lz77_compress(input_file, output_file, window_size).expect("LZ77 file error");

    return output_filepath;
}

fn get_filesize(filepath: &String) -> u64 {
    return metadata(filepath).expect("Couldn't get file metadata").len();
}

fn time_it<T, F>(func: F) -> (T, f64) where F: FnOnce() -> T {
    let start_t = std::time::Instant::now();
    let return_value = func();
    let end_t = std::time::Instant::now();
    return (return_value, end_t.duration_since(start_t).as_secs_f64());
}

fn calc_compr_ratio<F>(input_file: &String, func: F) -> f64 where F: FnOnce() -> String {
    let input_size = get_filesize(input_file);
    let output_file = func();
    let output_size = get_filesize(&output_file);
    return input_size as f64 / output_size as f64;
}

// TODO: 1. Add time, then store that into table, and in the end, display the table. Then, plot it using python.
// TODO: 2. Use multiple types of files
// TODO: 3. Maybe show an average as well

struct DataPoint {
    time: f64,
    compression_ratio: f64,
}

struct CompressionRow {
    window_size: usize,
    huffman: DataPoint,
    lz77: DataPoint,
    deflate: DataPoint,
}

type CompressionTable = Vec<CompressionRow>;

fn print_compression_table(table: &CompressionTable) {
    println!(
        "{:<16} | {:<10} {:<18} | {:<10} {:<18} | {:<10} {:<18}",
        "Window Size",
        "Time(s)", "Ratio (Huffman)",
        "Time(s)", "Ratio (LZ77)",
        "Time(s)", "Ratio (Deflate)"
    );
    println!("{}", "-".repeat(90));

    for (_, row) in table.iter().enumerate() {
        println!(
            "{:<16} | {:<10.4} {:<18.4} | {:<10.4} {:<18.4} | {:<10.4} {:<18.4}",
            row.window_size,
            row.huffman.time, row.huffman.compression_ratio,
            row.lz77.time, row.lz77.compression_ratio,
            row.deflate.time, row.deflate.compression_ratio
        );
    }
}

fn main() {
    let args :Vec<String> = std::env::args().collect();

    let filepath = &args[2];
    let mut table = CompressionTable::new();

    for shift_len in 2..=12 {
        let window_size = 1 << shift_len;

        println!("--- Window size: {} ---", &window_size);

        if args[1] == "compress" {
            // Huffman encoding
            let (time_taken, compression_ratio) = time_it(||
                calc_compr_ratio(
                    &filepath,
                    || compress_using_huffman(&filepath, "huff")
                )
            );
            let huffman_datapoint = DataPoint {
                time: time_taken,
                compression_ratio: compression_ratio,
            };
            println!("Huffman Encoding Complete");

            // let input_size = get_filesize(filepath);
            // let start_t = std::time::Instant::now();
            // let output_filepath = compress_using_huffman(filepath, "huff");
            // let end_t = std::time::Instant::now();
            // let time_taken = end_t.duration_since(start_t).as_secs_f64();
            // let output_size = get_filesize(&output_filepath);
            // let compression_ratio = input_size as f64 / output_size as f64;
            // let huffman_datapoint = DataPoint {
            //     time: time_taken,
            //     compression_ratio: compression_ratio,
            // };
            // println!("Huffman Encoding Complete");

            // LZ77 Compression
            let (time_taken, compression_ratio) = time_it(||
                calc_compr_ratio(
                    filepath,
                    || compress_using_lz77(filepath, window_size)
                )
            );
            let lz77_datapoint = DataPoint {
                time: time_taken,
                compression_ratio: compression_ratio,
            };
            println!("LZ77 Compression Complete");
            // compress_using_lz77(filepath, window_size);
            // println!("LZ77 Compression Complete");

            // Deflate
            let (time_taken, compression_ratio) = time_it(||
                calc_compr_ratio(
                    filepath,
                    || {
                        let output_file = compress_using_lz77(filepath, window_size);
                        return compress_using_huffman(&output_file, "defl");
                    }
                )
            );
            let defl_datapoint = DataPoint {
                time: time_taken,
                compression_ratio: compression_ratio,
            };
            println!("Deflate Compression Complete");

            let row = CompressionRow {
                window_size: window_size,
                huffman: huffman_datapoint,
                lz77: lz77_datapoint,
                deflate: defl_datapoint,
            };
            table.push(row);

            // compress_using_lz77(filepath, window_size);
            // compress_using_huffman(&format!("{}.lz77", filepath), "defl");
            // println!("Deflate Compression Complete");
        }
        else if args[1] == "decompress" {
            lz77_decompress(&mut File::open(args[2].as_str()).expect("Couldn't open compressed file."), &mut File::create(args[2].trim_end_matches(".z")).expect("Couldn't create output file"), window_size).expect("LZ77 file error");
        }
    }

    // Show the table properly formatted
    print_compression_table(&table);
}
