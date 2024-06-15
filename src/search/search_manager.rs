use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
    vec,
};

use arrayvec::ArrayVec;

use crate::{
    move_generation::{
        board_rep::{Board, Color, START_FEN},
        chess_move::Move,
        movegen::MovePicker,
    },
    search::constants::{
        Depth, EvalScore, Milliseconds, Nodes, Ply, EVAL_MAX, INF, MATE_THRESHOLD, MAX_DEPTH,
        MAX_PLY,
    },
    uci::setoption::Hash,
};

// temporary until I train my own net
fn temp_eval(board: &Board) -> EvalScore {
    let acc = crate::nnue::network::Accumulator::from_pos(board);
    acc.evaluate(board.stm)
}

use super::{
    history::History,
    killers::Killers,
    pv_table::PvTable,
    search_timer::SearchTimer,
    transposition_table::{TTFlag, TranspositionTable},
    zobrist_stack::ZobristStack,
};

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
    tt: TranspositionTable,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            searcher: Searcher::new(),
            board: Board::from_fen(START_FEN),
            tt: TranspositionTable::new(Hash::DEFAULT as usize),
        }
    }

    pub fn newgame(&mut self) {
        self.tt.reset_entries();
        self.searcher.history = History::new();
        self.searcher.killers = Killers::new();
    }

    pub fn resize_tt(&mut self, megabytes: u32) {
        self.tt = TranspositionTable::new(megabytes as usize);
    }

    pub fn update_state(&mut self, board: &Board, zobrist_stack: &ZobristStack) {
        self.board = board.clone();
        self.searcher.zobrist_stack = zobrist_stack.clone();
    }

    pub fn start_search(&mut self, config: &SearchConfig) {
        self.searcher.go(&self.board, &self.tt, config, true);
    }

    pub fn start_bench_search(&mut self, depth: Depth) -> Nodes {
        let mut config = SearchConfig::new(0);
        config.limits.push(SearchLimit::Depth(depth));

        clear_stop_flag();
        self.searcher.go(&self.board, &self.tt, &config, false);

        self.searcher.node_cnt
    }
}

#[derive(Debug, Clone)]
struct Searcher {
    timer: Option<SearchTimer>,
    zobrist_stack: ZobristStack,
    history: History,
    killers: Killers,

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
            history: History::new(),
            killers: Killers::new(),
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

    fn report_search_info(
        &self,
        tt: &TranspositionTable,
        score: EvalScore,
        depth: Depth,
        stopwatch: Instant,
    ) {
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
            "info score {score_str} time {time} nodes {} nps {nps} depth {depth} seldepth {} hashfull {} pv {}",
            self.node_cnt,
            self.seldepth,
            tt.hashfull(), // TODO: store hashfull somewhere, and only update it outside of searches (should give speedup)
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
            // TODO: replace with soft tm
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

    fn go(
        &mut self,
        board: &Board,
        tt: &TranspositionTable,
        config: &SearchConfig,
        report_info: bool,
    ) {
        self.reset_info();

        self.timer = None;
        self.init_search_timer(board.stm, config);

        let stopwatch = Instant::now();

        let mut best_move = Move::NULL;
        let mut depth = 1;
        while self.continue_deepening(config, depth) {
            let score = self.negamax::<true, true>(board, tt, depth, 0, -INF, INF);

            if stop_flag_is_set() {
                break;
            }

            if report_info {
                self.report_search_info(tt, score, depth, stopwatch);
            }

            best_move = self.pv_table.best_move();
            depth += 1;
        }
        set_stop_flag();

        if best_move.is_null() {
            eprintln!("WARNING: SEARCH RETURNED NULLMOVE");
            best_move = MovePicker::first_legal_mv(board).expect("NO LEGAL MOVES IN POSITION");
        }

        if report_info {
            println!("bestmove {}", best_move.as_string());
        }

        self.history.age_scores();
    }

    fn out_of_time(&self) -> bool {
        if self.node_cnt % Self::TIMER_CHECK_FREQ == 0 {
            if let Some(t) = self.timer {
                return t.is_hard_expired();
            }
        }

        false
    }

    fn negamax<const IS_ROOT: bool, const DO_NULL_MOVE: bool>(
        &mut self,
        board: &Board,
        tt: &TranspositionTable,
        depth: Depth,
        ply: Ply,
        mut alpha: EvalScore,
        beta: EvalScore,
    ) -> EvalScore {
        if IS_ROOT {
            self.seldepth = 0;
        }

        self.pv_table.set_length(ply);

        let old_alpha = alpha;
        let is_pv = beta != alpha + 1;
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

        self.seldepth = self.seldepth.max(ply);

        // PROBE TT
        let hash = self.zobrist_stack.current_hash();
        let tt_move = if let Some(tt_entry) = tt.probe(hash) {
            let tt_score = tt_entry.score_from_tt(ply);

            if !is_pv && tt_entry.cutoff_is_possible(alpha, beta, depth) {
                return tt_score;
            }

            tt_entry.mv
        } else {
            Move::NULL
        };

        let pruning_allowed = !is_pv && !in_check && alpha.abs() < MATE_THRESHOLD;

        let d = i32::from(depth);
        if pruning_allowed {
            // REVERSE FUTILITY PRUNING
            const RFP_MIN_DEPTH: Depth = 8;
            const RFP_MARGIN: EvalScore = 120;

            let static_eval = temp_eval(board);
            if depth <= RFP_MIN_DEPTH && static_eval >= (beta + RFP_MARGIN * d) {
                return static_eval;
            }

            // NULL MOVE PRUNING
            const NMP_MIN_DEPTH: Depth = 3;
            if DO_NULL_MOVE && depth >= NMP_MIN_DEPTH {
                // TODO: add zugzwang check
                let reduction = 3;

                let mut nmp_board = board.clone();
                nmp_board.play_nullmove(&mut self.zobrist_stack);
                let null_move_score = -self.negamax::<false, false>(
                    &nmp_board,
                    tt,
                    depth.saturating_sub(reduction),
                    ply + 1,
                    -beta,
                    -beta + 1,
                );

                self.zobrist_stack.pop();

                if null_move_score >= beta {
                    return null_move_score;
                }
            }
        }

        let mut best_score = -INF;
        let mut best_move = Move::NULL;
        let mut moves_played = 0;

        let mut move_picker = MovePicker::new();
        let mut played_quiets: ArrayVec<Move, { MovePicker::SIZE }> = ArrayVec::new();

        while let Some(mv) =
            move_picker.pick::<true>(board, &self.history, tt_move, self.killers.killer(ply))
        {
            let mut new_board = board.clone();

            let is_legal = new_board.try_play_move(mv, &mut self.zobrist_stack);
            if !is_legal {
                continue;
            }

            moves_played += 1;
            self.node_cnt += 1;

            #[allow(unused_assignments)]
            // TODO: maybe refactor this later idk
            let mut score = 0;
            if moves_played == 1 {
                score =
                    -self.negamax::<false, true>(&new_board, tt, depth - 1, ply + 1, -beta, -alpha);
            } else {
                // FULL DEPTH PVS
                score = -self.negamax::<false, true>(
                    &new_board,
                    tt,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                );

                // if our null-window search beat alpha without failing high, that means we might have a better move and need to re search with full window
                if score > alpha && score < beta {
                    score = -self.negamax::<false, true>(
                        &new_board,
                        tt,
                        depth - 1,
                        ply + 1,
                        -beta,
                        -alpha,
                    );
                }
            }

            self.zobrist_stack.pop();

            if stop_flag_is_set() || self.out_of_time() {
                set_stop_flag();
                return 0;
            }

            let is_quiet = mv.is_quiet();
            if is_quiet {
                played_quiets.push(mv);
            }

            if score > best_score {
                best_score = score;

                if score > alpha {
                    best_move = mv;
                    alpha = score;

                    self.pv_table.update(ply, mv);
                }

                if score >= beta {
                    if is_quiet {
                        self.killers.update(mv, ply);
                        self.history.update(board, played_quiets.as_slice(), depth);
                    }
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

        let tt_flag = TTFlag::determine(best_score, old_alpha, alpha, beta);
        tt.store(tt_flag, best_score, hash, ply, depth, best_move);
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

        let stand_pat = temp_eval(board);
        if stand_pat >= beta {
            return stand_pat;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut generator = MovePicker::new();

        let mut best_score = stand_pat;
        let mut _best_move = Move::NULL;
        while let Some(mv) = generator.simple_pick::<false>(board) {
            let mut next_board = board.clone();
            let is_legal = next_board.try_play_move(mv, &mut self.zobrist_stack);
            if !is_legal {
                continue;
            }

            self.node_cnt += 1;

            let score = -self.qsearch(&next_board, ply + 1, -beta, -alpha);

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
                }

                if score >= beta {
                    break;
                }
            }
        }

        best_score
    }
}
