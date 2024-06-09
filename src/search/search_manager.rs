use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
    vec,
};

use crate::{
    move_generation::{
        board_rep::{Board, Color, START_FEN},
        chess_move::Move,
        movegen::MovePicker,
    },
    nnue::eval::material_diff,
    search::constants::{
        Depth, EvalScore, Milliseconds, Nodes, Ply, EVAL_MAX, INF, MATE_THRESHOLD, MAX_DEPTH,
        MAX_PLY,
    },
};

use super::{pv_table::PvTable, search_timer::SearchTimer, zobrist_stack::ZobristStack};

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
    Infinite,
}

pub struct SearchConfig {
    pub limits: Vec<SearchLimit>,
    pub time: [Milliseconds; Color::CNT as usize],
    pub inc: [Milliseconds; Color::CNT as usize],
    pub overhead: Milliseconds,
    pub moves_to_go: Option<u32>,
}

impl SearchConfig {
    pub const fn new(overhead: Milliseconds) -> Self {
        Self {
            limits: vec![],
            time: [0, 0],
            inc: [0, 0],
            overhead,
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

    pub fn update_state(&mut self, board: &Board, zobrist_stack: &ZobristStack) {
        self.board = board.clone();
        self.searcher.zobrist_stack = zobrist_stack.clone();
    }

    pub fn start_search(&mut self, config: &SearchConfig) {
        self.searcher.go(&self.board, config, true);
    }

    pub fn start_bench_search(&mut self, depth: Depth) -> Nodes {
        let mut config = SearchConfig::new(0);
        config.limits.push(SearchLimit::Depth(depth));

        clear_stop_flag();
        self.searcher.go(&self.board, &config, false);

        self.searcher.node_cnt
    }
}

#[derive(Debug, Clone)]
struct Searcher {
    timer: Option<SearchTimer>,
    zobrist_stack: ZobristStack,

    // info
    pv_table: PvTable,
    best_move: Move,
    seldepth: u8,
    node_cnt: u64,
}

impl Searcher {
    const TIMER_CHECK_FREQ: u64 = 1024;

    fn new() -> Self {
        Self {
            timer: None,
            zobrist_stack: ZobristStack::new(&Board::from_fen(START_FEN)),
            pv_table: PvTable::new(),
            best_move: Move::NULL,
            seldepth: 0,
            node_cnt: 0,
        }
    }

    fn reset_info(&mut self) {
        self.best_move = Move::NULL;
        self.seldepth = 0;
        self.node_cnt = 0;
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
            "info score {score_str} time {time} nodes {} nps {nps} depth {depth} seldepth {} pv {}",
            self.node_cnt,
            self.seldepth,
            self.pv_table.pv_string()
        );
    }

    fn init_search_timer(&mut self, stm: Color, config: &SearchConfig) {
        let time = config.time[stm.as_index()];
        let inc = config.inc[stm.as_index()];

        for &limit in &config.limits {
            match limit {
                SearchLimit::Standard => {
                    let t = (time / 25 + inc / 2).saturating_sub(config.overhead);
                    self.timer = Some(SearchTimer::new(t));
                    break;
                }
                SearchLimit::MoveTime(time) => {
                    let t = time.saturating_sub(config.overhead);
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
            // TODO: replace with soft
            if timer.is_hard_expired() {
                return false;
            }
        }

        let mut result = true;
        for &limit in config.limits.iter() {
            result &= match limit {
                SearchLimit::Depth(depth_limit) => next_depth <= depth_limit,
                SearchLimit::Nodes(node_limit) => self.node_cnt <= node_limit,
                _ => true,
            }
        }

        result
    }

    fn go(&mut self, board: &Board, config: &SearchConfig, report_info: bool) {
        self.reset_info();

        self.timer = None;
        self.init_search_timer(board.stm, config);

        let stopwatch = Instant::now();

        let mut best_move = Move::NULL;
        let mut depth = 1;
        while self.continue_deepening(config, depth) {
            let score = self.negamax::<true>(board, depth, 0, -INF, INF);

            if stop_flag_is_set() {
                break;
            }

            if report_info {
                self.report_search_info(score, depth, stopwatch);
            }

            best_move = self.pv_table.best_move();
            depth += 1;
        }
        set_stop_flag();

        assert_ne!(best_move, Move::NULL);

        if report_info {
            println!("bestmove {}", best_move.as_string());
        }
    }

    fn out_of_time(&self) -> bool {
        if self.node_cnt % Self::TIMER_CHECK_FREQ == 0 {
            if let Some(t) = self.timer {
                return t.is_hard_expired();
            }
        }

        false
    }

    fn negamax<const IS_ROOT: bool>(
        &mut self,
        board: &Board,
        depth: Depth,
        ply: Ply,
        mut alpha: EvalScore,
        beta: EvalScore,
    ) -> EvalScore {
        if IS_ROOT {
            self.seldepth = 0;
        }

        self.pv_table.set_length(ply);
        self.seldepth = self.seldepth.max(ply);

        let in_check = board.in_check();
        let is_drawn =
            self.zobrist_stack.twofold_repetition(board.halfmoves) || board.fifty_move_draw();

        if !IS_ROOT {
            if is_drawn {
                return 0;
            }

            // MATE DISTANCE PRUNING
            let mate_alpha = alpha.max(i32::from(ply) - EVAL_MAX);
            let mate_beta = beta.min(EVAL_MAX - (i32::from(ply) + 1));
            if mate_alpha >= mate_beta {
                return mate_alpha;
            }
        }

        if depth == 0 || ply >= MAX_PLY {
            return self.qsearch(board, ply, alpha, beta);
        }

        let mut best_score = -INF;
        let mut _best_move = Move::NULL;

        let mut move_picker = MovePicker::new::<true>(board);
        let mut moves_played = 0;
        while let Some(mv) = move_picker.pick() {
            let mut new_board = board.clone();

            let is_legal = new_board.try_play_move(mv, &mut self.zobrist_stack);
            if !is_legal {
                continue;
            }

            moves_played += 1;
            self.node_cnt += 1;

            let score = -self.negamax::<false>(&new_board, depth - 1, ply + 1, -beta, -alpha);

            self.zobrist_stack.pop();

            if stop_flag_is_set() || self.out_of_time() {
                set_stop_flag();
                return 0;
            }

            if score > best_score {
                best_score = score;

                if score > alpha {
                    _best_move = mv;
                    alpha = score;

                    self.pv_table.update(ply, mv);
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

        // TODO: use best_move for tt here
        best_score
    }

    fn qsearch(
        &mut self,
        board: &Board,
        ply: Ply,
        mut alpha: EvalScore,
        beta: EvalScore,
    ) -> EvalScore {
        self.seldepth = self.seldepth.max(ply);

        let stand_pat = material_diff(board);
        if stand_pat >= beta {
            return stand_pat;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut generator = MovePicker::new::<false>(&board);

        let mut best_score = stand_pat;
        let mut _best_move = Move::NULL;
        while let Some(mv) =
            generator.pick()
        {
            let mut next_board = board.clone();
            let is_legal = next_board.try_play_move(mv, &mut self.zobrist_stack);
            if !is_legal {
                continue;
            }

            self.node_cnt += 1;

            let score = -self.qsearch(&next_board, ply + 1, -beta, -alpha);

            self.zobrist_stack.pop();

            if stop_flag_is_set() || self.out_of_time() { // TODO: try moving this above score
                set_stop_flag();
                return 0;
            }

            if score > best_score {
                best_score = score;

                if score > alpha {
                    _best_move = mv;
                    alpha = score;
                }

                if score >= beta {
                    break;
                }
            }
        }

        best_score
    }
}
