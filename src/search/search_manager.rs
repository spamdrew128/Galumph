use crate::movegen::{
    board_rep::{Board, START_FEN},
    chess_move::Move,
};

pub type Milliseconds = u128;
pub type Nodes = u64;
pub type Depth = i8;
pub type Ply = u8;
const MAX_DEPTH: Depth = i8::MAX;
pub const MAX_PLY: Ply = MAX_DEPTH as u8;

pub type EvalScore = i32;
pub const INF: EvalScore = (i16::MAX - 10) as i32;
pub const EVAL_MAX: EvalScore = INF - 1;
pub const MATE_THRESHOLD: EvalScore = EVAL_MAX - (MAX_PLY as i32);

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

    pub fn update_board(&mut self, board: &Board) {
        self.board = board.clone();
    }

    pub fn start_search(&mut self) {}
}

#[derive(Debug, Clone, Copy)]
struct Searcher {
    best_move: Move,

    // info
    seldepth: u8,
}

impl Searcher {
    fn new() -> Self {
        Self {
            best_move: Move::NULL,
            seldepth: 0,
        }
    }

    fn negamax(
        &mut self,
        board: &Board,
        mut depth: Depth,
        ply: Ply,
        mut alpha: EvalScore,
        beta: EvalScore,
    ) {
        
    }
}
