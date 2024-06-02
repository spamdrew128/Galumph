use crate::movegen::board_rep::{Color, Piece, Square};

const INPUT_SIZE: usize = Square::CNT as usize * Piece::CNT as usize * Color::CNT as usize;
const L1_SIZE: usize = 64;
const INPUT_SCALE: usize = 255;

static NNUE: Network =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/net.bin"))) };

#[repr(C, align(64))]
pub struct Accumulator {
    vals: [i16; L1_SIZE],
}

#[repr(C, align(64))]
pub struct L1Params {
    vals: [i16; L1_SIZE],
}

#[repr(C)]
pub struct Network {
    l1_weights: [L1Params; INPUT_SIZE],
    l1_biases: L1Params,
    output_weights: [L1Params; Color::CNT as usize],
    output_biases: i16,
}
