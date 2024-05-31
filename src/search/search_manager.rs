use std::{
    sync::atomic::{AtomicBool, Ordering}, time::Instant, vec
};

use crate::{
    evaluation::eval::material_diff,
    movegen::{
        board_rep::{Board, Color, START_FEN},
        chess_move::Move,
        movegen::MovePicker,
    },
    search::constants::{
        Depth, EvalScore, Milliseconds, Nodes, Ply, EVAL_MAX, INF, MATE_THRESHOLD, MAX_DEPTH,
        MAX_PLY,
    },
};

use super::search_timer::SearchTimer;

static STOP_FLAG: AtomicBool = AtomicBool::new(false);

pub fn stop_flag_is_set() -> bool {
    STOP_FLAG.load(Ordering::Relaxed)
}

pub fn set_stop_flag() {
    STOP_FLAG.store(true, Ordering::Relaxed);
}

pub fn clear_stop_flag() {
    STOP_FLAG.store(false, Ordering::Relaxed);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchLimit {
    Standard,
    MoveTime(Milliseconds),
    Depth(Depth),
    Nodes(Nodes),
}

pub struct SearchConfig {
    pub limits: Vec<SearchLimit>,
    pub time: [Milliseconds; Color::CNT as usize],
    pub inc: [Milliseconds; Color::CNT as usize],
    pub moves_to_go: Option<u32>,
}

impl SearchConfig {
    pub const fn new() -> Self {
        Self {
            limits: vec![],
            time: [0, 0],
            inc: [0, 0],
            moves_to_go: None,
        }
    }
}

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

    pub fn start_search(&mut self, config: &SearchConfig) {
        self.searcher.go(&self.board, config);
    }
}

#[derive(Debug, Clone, Copy)]
struct Searcher {
    timer: Option<SearchTimer>,

    // info
    best_move: Move,
    seldepth: u8,
    node_cnt: u64,
}

impl Searcher {
    const TIMER_CHECK_FREQ: u64 = 1024;

    fn new() -> Self {
        Self {
            timer: None,
            best_move: Move::NULL,
            seldepth: 0,
            node_cnt: 0,
        }
    }

    fn report_search_info(&self, score: EvalScore, depth: Depth, stopwatch: Instant) {
        let score_str = if score >= MATE_THRESHOLD {
            let ply = EVAL_MAX - score;
            let score_value = (ply + 1) / 2;

            format!("mate {score_value}")
        } else if score <= -MATE_THRESHOLD {
            let ply = EVAL_MAX + score;
            let score_value = (ply + 1) / 2;

            format!("mate -{score_value}")
        } else {
            format!("cp {score}")
        };

        let elapsed = stopwatch.elapsed();
        let time = elapsed.as_millis();
        let nps = (u128::from(self.node_cnt) * 1_000_000) / elapsed.as_micros().max(1);

        println!(
            "info score {score_str} time {time} nodes {} nps {nps} depth {depth} seldepth {}",
            self.node_cnt,
            self.seldepth
        );
    }

    fn init_search_timer(&mut self, stm: Color, config: &SearchConfig) {
        let time = config.time[stm.as_index()];
        let inc = config.inc[stm.as_index()];

        for &limit in &config.limits {
            match limit {
                SearchLimit::Standard => {
                    let t = time / 25 + inc / 2;
                    self.timer = Some(SearchTimer::new(t));
                    break;
                }
                SearchLimit::MoveTime(t) => {
                    self.timer = Some(SearchTimer::new(t));
                    break;
                }
                _ => (),
            }
        }
    }

    fn continue_deepening(&self, config: &SearchConfig, next_depth: Depth) -> bool {
        if next_depth == MAX_DEPTH {
            return false;
        }

        if let Some(timer) = self.timer {
            if timer.is_hard_expired() { // TODO: replace with soft
                return false;
            }
        }

        let mut result = true;
        for &limit in config.limits.iter() {
            result &= match limit {
                SearchLimit::Depth(depth_limit) => next_depth > depth_limit,
                SearchLimit::Nodes(node_limit) => self.node_cnt > node_limit,
                _ => true,
            }
        }

        result
    }

    fn go(&mut self, board: &Board, config: &SearchConfig) {
        self.timer = None;
        self.init_search_timer(board.stm, config);

        let stopwatch = Instant::now();

        let mut best_move = Move::NULL;
        let mut depth = 1;
        while self.continue_deepening(config, depth) {
            let score = self.negamax(board, depth, 0, -INF, INF);

            if stop_flag_is_set() {
                break;
            }

            self.report_search_info(score, depth, stopwatch);

            best_move = self.best_move;
            depth += 1;
        }

        assert_ne!(best_move, Move::NULL);
        println!("bestmove {}", best_move.as_string());

        set_stop_flag();
    }

    fn out_of_time(&self) -> bool {
        if self.node_cnt % Self::TIMER_CHECK_FREQ == 0 {
            if let Some(t) = self.timer {
                return t.is_hard_expired();
            }
        }

        false
    }

    fn negamax(
        &mut self,
        board: &Board,
        depth: Depth,
        ply: Ply,
        mut alpha: EvalScore,
        beta: EvalScore,
    ) -> EvalScore {
        self.seldepth = self.seldepth.max(ply);

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
            self.node_cnt += 1;

            let score = -self.negamax(&new_board, depth - 1, ply + 1, -beta, -alpha);

            if stop_flag_is_set() || self.out_of_time() {
                set_stop_flag();
                return 0;
            }

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
