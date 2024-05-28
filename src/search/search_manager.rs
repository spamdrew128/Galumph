use crate::movegen::{
    board_rep::{Board, START_FEN},
    chess_move::Move, movegen::MovePicker,
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
    ) -> EvalScore {
        if depth == 0 || ply >= MAX_PLY {
            return 0; // todo! eval
        }

        let mut best_score = -INF;
        let mut best_move = Move::NULL;

        let mut move_picker = MovePicker::new(board);
        while let Some(mv) = move_picker.pick() {
            let mut new_board = board.clone();

            let is_legal = new_board.try_play_move(mv);
            if !is_legal {
                continue;
            }

            let score = self.negamax(&new_board, depth, ply, alpha, beta)
        }

        best_score
    }
}
