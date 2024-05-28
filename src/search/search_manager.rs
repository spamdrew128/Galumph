use crate::{evaluation::eval::material_diff, movegen::{
    board_rep::{Board, START_FEN},
    chess_move::Move,
    movegen::MovePicker,
}, search::constants::{Depth, EvalScore, Ply, EVAL_MAX, INF, MAX_PLY}};

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

    pub fn start_search(&mut self) {
        let _score = self.searcher.negamax(&self.board, 6, 0, -INF, INF);
        println!("bestmove {}", self.searcher.best_move.as_string());
    }
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
        depth: Depth,
        ply: Ply,
        mut alpha: EvalScore,
        beta: EvalScore,
    ) -> EvalScore {
        if depth == 0 || ply >= MAX_PLY {
            return material_diff(board);
        }

        let in_check = board.in_check();

        let mut best_score = -INF;
        let mut best_move = Move::NULL;

        let mut move_picker = MovePicker::new(board);
        let mut moves_played = 0;
        while let Some(mv) = move_picker.pick() {
            let mut new_board = board.clone();

            let is_legal = new_board.try_play_move(mv);
            if !is_legal {
                continue;
            }
            moves_played += 1;

            let score = -self.negamax(&new_board, depth - 1, ply + 1, -beta, -alpha);

            if score > best_score {
                best_score = score;

                if score > alpha {
                    best_move = mv;
                    alpha = score;
                }

                if score >= beta {
                    break;
                }
            }
        }

        if moves_played == 0 {
            // either checkmate or stalemate
            return if in_check {
                -EVAL_MAX + i32::from(ply)
            } else {
                0
            };
        }

        self.best_move = best_move; // remove this later when we have PV table
        best_score
    }
}
