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

struct TimeAndComprRatio {
    time: f64,
    compression_ratio: f64,
}

fn time_it_and_calc_compr_ratio<F>(input_file: &String, func: F) -> TimeAndComprRatio where F: FnOnce() -> String {
    let input_size = get_filesize(input_file);
    let start_time = std::time::Instant::now();
    let output_file = func();
    let end_time = std::time::Instant::now();
    let output_size = get_filesize(&output_file);
    return TimeAndComprRatio {
        time: end_time.duration_since(start_time).as_secs_f64(),
        compression_ratio: input_size as f64 / output_size as f64
    };
}

// TODO: 1. Add time, then store that into table, and in the end, display the table. Then, plot it using python.
// TODO: 2. Use multiple types of files
// TODO: 3. Maybe show an average as well

struct DataPoint {
    time: f64,
    first_cr: f64,
    second_cr: f64,
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
        "{:<16} | {:<10} {:<18} | {:<10} {:<18} | {:<10} {:<18} ({:<8.8} x {:<8.8})",
        "Window Size",
        "Time(s)", "Ratio (Huffman)",
        "Time(s)", "Ratio (LZ77)",
        "Time(s)", "Ratio (Deflate)", "First CR",  "Second CR"
    );
    println!("{}", "-".repeat(140));

    for (_, row) in table.iter().enumerate() {
        println!(
            "{:<16} | {:<10.4} {:<18.4} | {:<10.4} {:<18.4} | {:<10.4} {:<18.4} ({:<8.4} x {:<8.4})",
            row.window_size,
            row.huffman.time, row.huffman.compression_ratio,
            row.lz77.time, row.lz77.compression_ratio,
            row.deflate.time, row.deflate.compression_ratio, row.deflate.first_cr, row.deflate.second_cr
        );
    }
}

fn to_csv(value: f64) -> String {
    format!("{:.4}", value)
}

fn write_to_csv(table: &CompressionTable, filename: &str) {
    let mut writer = csv::Writer::from_path(filename).expect("Couldn't create writer");

    writer.write_record(&["Window Size", "Huffman Time", "Huffman Ratio", "LZ77 Time", "LZ77 Ratio", "Deflate Time", "Deflate Ratio", "First CR", "Second CR"]).expect("Couldn't write to csv");


    for row in table.iter() {
        writer.write_record(&[row.window_size.to_string(),
            to_csv(row.huffman.time), to_csv(row.huffman.compression_ratio),
            to_csv(row.lz77.time), to_csv(row.lz77.compression_ratio),
            to_csv(row.deflate.time), to_csv(row.deflate.compression_ratio), to_csv(row.deflate.first_cr), to_csv(row.deflate.second_cr)
        ]).expect("Couldn't write to csv");
    }
}

const MAX_WINDOW_SHIFT: usize= 14;

fn run_on_file(filepath: &String, command: &String) {
    let mut table = CompressionTable::new();

    for shift_len in 2..=MAX_WINDOW_SHIFT {
        let window_size = 1 << shift_len;

        println!("--- Window size: {} ---", &window_size);

        if command == "compress" {
            // Huffman encoding
            let compr_ratio = time_it_and_calc_compr_ratio(
                &filepath,
                || compress_using_huffman(&filepath, "huff")
            );
            let time_taken = compr_ratio.time;
            let compression_ratio = compr_ratio.compression_ratio;
            let huffman_datapoint = DataPoint {
                time: time_taken,
                first_cr: 0f64,
                second_cr: 0f64,
                compression_ratio: compression_ratio,
            };
            println!("Huffman Encoding Complete");

            // LZ77 Compression
            let compr_ratio = time_it_and_calc_compr_ratio(
                &filepath,
                || compress_using_lz77(filepath, window_size)
            );
            let time_taken = compr_ratio.time;
            let compression_ratio = compr_ratio.compression_ratio;
            let lz77_datapoint = DataPoint {
                time: time_taken,
                first_cr: 0f64,
                second_cr: 0f64,
                compression_ratio: compression_ratio,
            };
            println!("LZ77 Compression Complete");

            // Deflate
            let compr_ratio = time_it_and_calc_compr_ratio(
                &filepath,
                || {
                    let output_file = compress_using_lz77(filepath, window_size);
                    return compress_using_huffman(&output_file, "defl");
                }
            );
            let input_filesize = get_filesize(filepath) as usize;
            let lz77_filesize = get_filesize(&format!("{}.lz77", filepath)) as usize;
            let defl_filesize = get_filesize(&format!("{}.lz77.defl", filepath)) as usize;
            let first_cr = input_filesize as f64 / lz77_filesize as f64;
            let second_cr = lz77_filesize as f64 / defl_filesize as f64;

            println!("Deflate Compression: {}, {}, {}, {}, {}", input_filesize, lz77_filesize, defl_filesize, first_cr, second_cr);

            let time_taken = compr_ratio.time;
            let compression_ratio = compr_ratio.compression_ratio;
            let defl_datapoint = DataPoint {
                time: time_taken,
                first_cr: first_cr,
                second_cr: second_cr,
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
        }
        else if command == "decompress" {
            lz77_decompress(&mut File::open(filepath.as_str()).expect("Couldn't open compressed file."), &mut File::create(filepath.trim_end_matches(".z")).expect("Couldn't create output file"), window_size).expect("LZ77 file error");
        }
    }

    // Show the table properly formatted
    print_compression_table(&table);

    // Write to CSV
    write_to_csv(&table, format!("compression_table_{}.csv", filepath).as_str());
}

fn main() {
    let args :Vec<String> = std::env::args().collect();

    // let filepath = &args[2];
    let filepaths: [&str; 1] = ["./analysis_files/CLRS-3rd.pdf"];

    for filepath in filepaths {
        run_on_file(&filepath.to_string(), &args[1]);
    }
}

// TODO:
// 1. Run on multiple files (CLRS.pdf book and an uncompressed PNG file)
// 2. Do decompress fully
