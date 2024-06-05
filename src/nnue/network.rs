use std::ops::{Index, IndexMut};

use crate::{
    bitloop,
    movegen::board_rep::{Board, Color, Piece, Square},
};

const INPUT_SIZE: usize = Square::CNT as usize * Piece::CNT as usize * Color::CNT as usize;
const L1_SIZE: usize = 768;

const INPUT_SCALE: i16 = 255;
const OUTPUT_SCALE: i16 = 64;

static NNUE: Network =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/net.bin"))) };

#[repr(C, align(64))]
pub struct L1Params([i16; L1_SIZE]); // TODO: Add index operator overloading.

#[repr(C)]
pub struct Network {
    l1_weights: [L1Params; INPUT_SIZE],
    l1_biases: L1Params,
    output_weights: [L1Params; Color::CNT as usize],
    output_bias: i16,
}

#[derive(Clone)]
pub struct FeatureIndexs([usize; Color::CNT as usize]);

impl FeatureIndexs {
    fn get(sq: Square, piece: Piece, piece_color: Color) -> Self {
        let color_stride = usize::from(Piece::CNT * Square::CNT);
        let piece_stride = usize::from(Square::CNT);

        let p = piece.as_nnue_index() * piece_stride;

        let white_idx = piece_color.as_index() * color_stride + p + sq.as_index();
        let black_idx = piece_color.flip().as_index() * color_stride + p + sq.mirror().as_index();

        Self([white_idx, black_idx])
    }
}

#[derive(Clone)]
#[repr(C, align(64))]
pub struct Accumulator([[i16; L1_SIZE]; Color::CNT as usize]);

impl Index<usize> for Accumulator {
    type Output = [i16; L1_SIZE];

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for Accumulator {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl Accumulator {
    pub const REMOVE: i16 = -1;
    pub const ADD: i16 = 1;

    fn new() -> Self {
        Self([[0; L1_SIZE]; Color::CNT as usize])
    }

    fn from_pos(board: &Board) -> Self {
        let mut res = Self::new();

        for color in Color::LIST {
            for piece in Piece::LIST {
                bitloop!(|sq| board.piece_bb(piece, color), {
                    let idxs = FeatureIndexs::get(sq, piece, color);
                    res.update::<{ Accumulator::ADD }>(&idxs)
                });
            }
        }

        res
    }

    fn update<const SIGN: i16>(&mut self, idxs: &FeatureIndexs) {
        for (&color, &idx) in Color::LIST.iter().zip(idxs.0.iter()) {
            let weights = &NNUE.l1_weights[idx].0;
            let acc = &mut self[color.as_index()];

            for (neuron_sum, &weight) in acc.iter_mut().zip(weights) {
                *neuron_sum += weight * SIGN;
            }
        }
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
