use crate::movegen::board_rep::{Board, Color, Piece, Square};

const INPUT_SIZE: usize = Square::CNT as usize * Piece::CNT as usize * Color::CNT as usize;
const L1_SIZE: usize = 768;

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

fn params_indexes(sq: Square, piece: Piece, color: Color) -> (usize, usize) {
    let color_stride = usize::from(Piece::CNT * Square::CNT);
    let piece_stride = usize::from(Square::CNT);

    let p = piece.as_nnue_index();

    let us_idx = color.as_index() * color_stride + p * piece_stride + sq.as_index();
    
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

        res
    }
}

#[cfg(test)]
mod tests {
    use super::NNUE;

    #[ignore]
    #[test]
    fn peep() {
        let _nnue = &NNUE;
        println!("hey");
    }
}