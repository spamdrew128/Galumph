use build_script_stuff::magic_builder::get_magic_bytes;
use build_script_stuff::nnue_bin_encoder::get_random_nnue_bytes;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

mod build_script_stuff;

fn gen_output_file(name: &str, buf: &[u8]) {
    let mut out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    out_dir.push(name);

    let mut out_file = BufWriter::new(File::create(out_dir).unwrap());
    out_file.write(buf).unwrap();
}

fn get_existing_net_bytes() -> Vec<u8> {
    let mut path = std::env::current_dir().unwrap();
    path.push("net_binary");
    path.push("net.bin");

    if path.is_file() {
        return std::fs::read(path).unwrap();
    }

    vec![]
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    // Magic table generation
    let magic_bytes = get_magic_bytes();
    gen_output_file("magic_init.bin", magic_bytes.as_slice());

    // NNUE file generation
    let existing_nnue_bytes = get_existing_net_bytes();

    if existing_nnue_bytes.is_empty() {
        let nnue_bytes = get_random_nnue_bytes();
        gen_output_file("net.bin", nnue_bytes.bytes.as_slice());
    } else {
        gen_output_file("net.bin", existing_nnue_bytes.as_slice());
    }
}
