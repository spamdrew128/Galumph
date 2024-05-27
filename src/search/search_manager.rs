use crate::movegen::board_rep::{Board, START_FEN};

pub struct SearchManager {
    board: Board,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            board: Board::from_fen(START_FEN),
        }
    }
}
