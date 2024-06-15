use build_script_stuff::lmr_builder::get_lmr_bytes;
use build_script_stuff::magic_builder::get_magic_bytes;
use build_script_stuff::zobrist_builder::get_zobrist_bytes;

use std::fs::File;
use std::fs::ReadDir;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

mod build_script_stuff;

fn gen_output_file(name: &str, buf: &[u8]) {
    let mut out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    out_dir.push(name);

    let mut out_file = BufWriter::new(File::create(out_dir).unwrap());
    out_file.write_all(buf).unwrap();
}

fn copy_file(from_path: PathBuf, name: &str) -> Result<u64, std::io::Error> {
    let mut out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    out_dir.push(name);

    std::fs::copy(from_path, out_dir)
}

fn copy_net_to_out_dir() {
    fn try_from_dir(dir_paths: &mut ReadDir) -> bool {
        while let Some(Ok(dir_entry)) = dir_paths.next() {
            let path = dir_entry.path();

            let mut header_path = path.clone();
            header_path.pop();
            header_path.push("header.rs");

            if path == header_path {
                continue;
            }

            if let Ok(bytes_read) = copy_file(path, "net.bin") {
                if bytes_read == 0 {
                    continue;
                }

                copy_file(header_path, "header.rs")
                    .expect("\n!!!! EXPECTED VALID HEADER FILE !!!!\n\n");

                return true;
            }
        }

        false
    }

    // try from user net directory
    let mut dir = std::env::current_dir().unwrap();
    dir.push("net_binaries");
    dir.push("user");

    let dir_paths = std::fs::read_dir(dir.clone());

    if let Ok(mut paths) = dir_paths {
        if try_from_dir(&mut paths) {
            return;
        }
    }

    // since user was empty, try from default directory
    dir.pop();
    dir.push("default");
    let dir_paths = std::fs::read_dir(dir);

    if let Ok(mut paths) = dir_paths {
        if try_from_dir(&mut paths) {
            return;
        }
    }

    panic!("NO VALID NET FILES FOUND!");
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    // Magic table generation
    let magic_bytes = get_magic_bytes();
    gen_output_file("magic_init.bin", magic_bytes.as_slice());

    // LMR table generation
    let lmr_bytes = get_lmr_bytes();
    gen_output_file("lmr_init.bin", lmr_bytes.as_slice());

    // Zobrist key generation
    let zobrist_bytes = get_zobrist_bytes();
    gen_output_file("zobrist_init.bin", zobrist_bytes.as_slice());

    // Copy NNUE file
    copy_net_to_out_dir();
}
