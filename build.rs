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

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let magic_bytes = get_magic_bytes();
    gen_output_file("magic_init.bin", magic_bytes.as_slice());

    let nnue_bytes = get_random_nnue_bytes();
    gen_output_file("net.bin", nnue_bytes.bytes.as_slice());
}
