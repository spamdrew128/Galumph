use crate::{
    movegen::board_rep::{Board, Piece},
    search::constants::EvalScore,
};

pub fn material_diff(board: &Board) -> EvalScore {
    let scores: [EvalScore; Piece::CNT as usize] = [300, 310, 500, 900, 100, 0];

    let stm = board.stm;
    let mut res = 0;
    for (&pc, &weight) in Piece::LIST.iter().zip(scores.iter()) {
        let diff = board.piece_bb(pc, stm).popcount() - board.piece_bb(pc, stm.flip()).popcount();
        res += i32::from(diff) * weight;
    }
    res
}
