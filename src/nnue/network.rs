use crate::{bitloop, movegen::board_rep::{Board, Color, Piece, Square}};

const INPUT_SIZE: usize = Square::CNT as usize * Piece::CNT as usize * Color::CNT as usize;
const L1_SIZE: usize = 64;
const INPUT_SCALE: i16 = 255;
const OUTPUT_SCALE: i16 = 64;

static NNUE: Network =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/net.bin"))) };

#[repr(C, align(64))]
pub struct L1Params([i16; L1_SIZE]);

#[repr(C)]
pub struct Network {
    l1_weights: [L1Params; INPUT_SIZE],
    l1_biases: L1Params,
    output_weights: [L1Params; Color::CNT as usize],
    output_bias: i16,
}

#[derive(Clone)]
#[repr(C, align(64))]
pub struct Accumulator([[i16; L1_SIZE]; Color::CNT as usize]);

impl Accumulator {
    fn new() -> Self {
        Self([[0; L1_SIZE]; Color::CNT as usize])
    }

    fn from_pos(board: &Board) -> Self {
        let mut res = Self::new();
        for p in Piece::LIST {
            let w_pieces = board.piece_bb(p, board.stm);
            let b_pieces = board.piece_bb(p, board.stm.flip());

            bitloop!(|sq| w_pieces, {
                res
            });
        }

        res
    }
}
