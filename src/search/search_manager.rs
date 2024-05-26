use crate::movegen::board_rep::Board;

pub struct SearchManager {
    board: Board,
}

impl SearchManager {
    pub fn new(fen: &str) -> Self {
        Self {
            board: Board::from_fen(fen),
        }
    }
}
