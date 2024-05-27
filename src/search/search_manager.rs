use crate::movegen::board_rep::{Board, START_FEN};

pub type Milliseconds = u128;
pub type Nodes = u64;
pub type Depth = i8;
pub type Ply = u8;
const MAX_DEPTH: Depth = i8::MAX;
pub const MAX_PLY: Ply = MAX_DEPTH as u8;

pub struct SearchManager {
    searcher: Searcher,
    board: Board,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            searcher: Searcher::new(),
            board: Board::from_fen(START_FEN),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Searcher {
    // info
    seldepth: u8,
}

impl Searcher {
    fn new() -> Self {
        Self { seldepth: 0 }
    }
}
