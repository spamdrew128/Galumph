use std::ops::{Index, IndexMut};

use crate::{
    bitloop,
    move_generation::board_rep::{Board, Color, Piece, Square},
    search::constants::EvalScore,
};

// TODO: put these in some sort of header file or library :)
const INPUT_SIZE: usize = Square::CNT as usize * Piece::CNT as usize * Color::CNT as usize;
const L1_SIZE: usize = 64;

const L1_SCALE: i16 = 255;
const OUTPUT_SCALE: i16 = 64;

fn activation(sum: i16) -> i32 {
    i32::from(sum.clamp(0, L1_SCALE))
}

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
pub struct FeatureIndices([usize; Color::CNT as usize]);

impl FeatureIndices {
    fn get(sq: Square, piece: Piece, piece_color: Color) -> Self {
        let color_stride = usize::from(Piece::CNT) * usize::from(Square::CNT);
        let piece_stride = usize::from(Square::CNT);

        let p = piece.as_nnue_index() * piece_stride;

        // white index uses sq.mirror() because I use board representation layout
        // that is mirrored from the standard layout, so some translation is required
        // to convert to a feature index.
        let white_idx = piece_color.as_index() * color_stride + p + sq.mirror().as_index();
        let black_idx = piece_color.flip().as_index() * color_stride + p + sq.as_index();

        Self([white_idx, black_idx])
    }
}

#[derive(Debug, Clone)]
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
        Self([NNUE.l1_biases.0; Color::CNT as usize])
    }

    pub fn from_pos(board: &Board) -> Self {
        let mut res = Self::new();

        for color in Color::LIST {
            for piece in Piece::LIST {
                bitloop!(|sq| board.piece_bb(piece, color), {
                    let idxs = FeatureIndices::get(sq, piece, color);
                    res.update::<{ Accumulator::ADD }>(&idxs)
                });
            }
        }

        res
    }

    pub fn update<const SIGN: i16>(&mut self, idxs: &FeatureIndices) {
        for (acc, &idx) in self.0.iter_mut().zip(idxs.0.iter()) {
            let weights = &NNUE.l1_weights[idx].0;

            for (neuron_sum, &weight) in acc.iter_mut().zip(weights) {
                *neuron_sum += weight * SIGN;
            }
        }
    }

    pub fn evaluate(&self, stm: Color) -> EvalScore {
        let (us, them) = (stm.as_index(), stm.flip().as_index());

        let our_sums = self[us].iter();
        let our_weights = &NNUE.output_weights[0].0;

        let their_sums = self[them].iter();
        let their_weights = &NNUE.output_weights[1].0;

        let mut eval = 0;

        for (&sum, &weight) in our_sums.zip(our_weights) {
            eval += activation(sum) * i32::from(weight);
        }
        for (&sum, &weight) in their_sums.zip(their_weights) {
            eval += activation(sum) * i32::from(weight);
        }

        eval += EvalScore::from(NNUE.output_bias);

        (eval * 400) / i32::from(L1_SCALE * OUTPUT_SCALE)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        move_generation::board_rep::{Board, START_FEN},
        nnue::network::Accumulator,
    };

    use super::NNUE;

    #[test]
    fn peep() {
        let _nnue = &NNUE;

        let board = Board::from_fen(START_FEN);
        let acc = Accumulator::from_pos(&board);
        // println!("{:?}", acc);
        let eval = acc.evaluate(board.stm);
        println!("{eval}");
    }
}
