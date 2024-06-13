use crate::{
    bitloop,
    move_generation::{
        attacks,
        board_rep::{Bitboard, Board, Piece, Square},
        chess_move::{Flag, Move},
    },
    search::history::History,
    tuple_constants_enum,
};

macro_rules! into_moves {
    (|$from:ident| $piece_bb:ident, |$to:ident| $moves_bb:expr, $add_move:expr) => {{
        bitloop!(|$from| $piece_bb, {
            let moves: Bitboard = $moves_bb;
            bitloop!(|$to| moves, { $add_move });
        });
    }};
}

const MVV_LVA: [[i16; (Piece::CNT + 1) as usize]; (Piece::CNT + 1) as usize] = {
    // knight, bishop, rook, queen, pawn, king, none (for en passant)
    let scores: [i16; (Piece::CNT + 1) as usize] = [3, 4, 5, 9, 1, 0, 1];
    let mut result: [[i16; (Piece::CNT + 1) as usize]; (Piece::CNT + 1) as usize] =
        [[0; (Piece::CNT + 1) as usize]; (Piece::CNT + 1) as usize];

    let mut a = 0;
    while a < (Piece::CNT + 1) as usize {
        let mut v = 0;
        while v < (Piece::CNT + 1) as usize {
            result[a][v] = scores[v] - scores[a];
            v += 1;
        }
        a += 1;
    }

    result
};

const fn mvv_lva(attacker: Piece, victim: Piece) -> i16 {
    MVV_LVA[attacker.as_index()][victim.as_index()]
}

#[derive(Debug, Copy, Clone)]
pub struct ScoredMove {
    mv: Move,
    score: i16,
}

impl ScoredMove {
    const EMPTY: Self = Self::new();

    const fn new() -> Self {
        Self {
            mv: Move::NULL,
            score: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MoveStage(u8);

impl MoveStage {
    #[rustfmt::skip]
    tuple_constants_enum!(Self,
        START,
        TT_MOVE,
        NOISY,
        KILLER,
        QUIET
    );

    const fn new(data: u8) -> Self {
        Self(data)
    }

    fn increment(&mut self) {
        self.0 += 1;
    }
}

pub struct MovePicker {
    list: [ScoredMove; Self::SIZE],
    stage: MoveStage,
    idx: usize,
    limit: usize,
}

impl MovePicker {
    pub const SIZE: usize = u8::MAX as usize;

    pub fn new() -> Self {
        Self {
            list: [ScoredMove::EMPTY; Self::SIZE],
            stage: MoveStage::START,
            idx: 0,
            limit: 0,
        }
    }

    fn add(&mut self, mv: Move) {
        self.list[self.limit].mv = mv;
        self.limit += 1;
    }

    const fn stage_complete(&self) -> bool {
        self.idx >= self.limit
    }

    fn advance_stage(&mut self) {
        self.stage.increment();
        self.idx = self.limit;
    }

    fn gen_moves<const NOISY: bool>(&mut self, board: &Board) {
        let opps = board.them();
        let occ = board.occupied();

        let (filter, flag) = if NOISY {
            (opps, Flag::CAPTURE)
        } else {
            (!occ, Flag::NONE)
        };

        let stm = board.stm;
        let knights = board.piece_bb(Piece::KNIGHT, stm);
        let bishops = board.piece_bb(Piece::BISHOP, stm);
        let rooks = board.piece_bb(Piece::ROOK, stm);
        let queens = board.piece_bb(Piece::QUEEN, stm);
        let king = board.piece_bb(Piece::KING, stm);

        into_moves!(|from| knights, |to| attacks::knight(from).and(filter), {
            self.add(Move::new(to, from, flag));
        });
        into_moves!(
            |from| bishops,
            |to| attacks::bishop(from, occ).and(filter),
            {
                self.add(Move::new(to, from, flag));
            }
        );
        into_moves!(|from| rooks, |to| attacks::rook(from, occ).and(filter), {
            self.add(Move::new(to, from, flag));
        });
        into_moves!(|from| queens, |to| attacks::queen(from, occ).and(filter), {
            self.add(Move::new(to, from, flag));
        });
        into_moves!(|from| king, |to| attacks::king(from).and(filter), {
            self.add(Move::new(to, from, flag));
        });

        let pawns = board.piece_bb(Piece::PAWN, stm);
        let promo_pawns = board.promotable_pawns();
        let normal_pawns = pawns & !promo_pawns;

        into_moves!(
            |from| promo_pawns,
            |to| attacks::pawn(from, stm).and(opps),
            {
                if NOISY {
                    self.add(Move::new(to, from, Flag::QUEEN_CAPTURE_PROMO));
                } else {
                    self.add(Move::new(to, from, Flag::KNIGHT_CAPTURE_PROMO));
                    self.add(Move::new(to, from, Flag::BISHOP_CAPTURE_PROMO));
                    self.add(Move::new(to, from, Flag::ROOK_CAPTURE_PROMO));
                }
            }
        );

        let promotion_moves = attacks::pawn_single_push(promo_pawns, occ, stm);
        bitloop!(|to| promotion_moves, {
            let from = to.retreat(1, stm);
            if NOISY {
                self.add(Move::new(to, from, Flag::QUEEN_PROMO));
            } else {
                self.add(Move::new(to, from, Flag::KNIGHT_PROMO));
                self.add(Move::new(to, from, Flag::BISHOP_PROMO));
                self.add(Move::new(to, from, Flag::ROOK_PROMO));
            }
        });

        if NOISY {
            into_moves!(
                |from| normal_pawns,
                |to| attacks::pawn(from, stm).and(opps),
                {
                    self.add(Move::new(to, from, Flag::CAPTURE));
                }
            );

            if let Some(ep_sq) = board.ep_sq {
                let attackers = attacks::pawn(ep_sq, stm.flip()) & pawns;
                bitloop!(|from| attackers, {
                    self.add(Move::new(ep_sq, from, Flag::EP));
                });
            }
        } else {
            let single_pushs = attacks::pawn_single_push(normal_pawns, occ, stm);
            let double_pushes = attacks::pawn_double_push(single_pushs, occ, stm);

            bitloop!(|to| single_pushs, {
                let from = to.retreat(1, stm);
                self.add(Move::new(to, from, flag));
            });

            bitloop!(|to| double_pushes, {
                let from = to.double_push_sq();
                self.add(Move::new(to, from, Flag::DOUBLE_PUSH));
            });

            let king_sq = board.king_sq();
            if board.can_ks_castle() {
                self.add(Move::new_ks_castle(king_sq))
            }
            if board.can_qs_castle() {
                self.add(Move::new_qs_castle(king_sq))
            }
        }
    }

    // Essentially this is selection sort
    fn next_best_move(&mut self) -> Move {
        let mut best_idx = self.idx;
        let mut best_score = self.list[self.idx].score;
        for i in (self.idx + 1)..self.limit {
            let score = self.list[i].score;
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        let mv = self.list[best_idx].mv;
        self.list.swap(self.idx, best_idx);
        self.idx += 1;
        mv
    }

    fn score_noisy_moves(&mut self, board: &Board) {
        // TODO: give bonus to promotions
        let mut start = self.idx as i32;
        let end = self.limit as i32 - 1;

        while start <= end {
            let i = start as usize;
            let mv = self.list[i].mv;

            let attacker = board.piece_on_sq(mv.from());
            let victim = board.piece_on_sq(mv.to());
            self.list[i].score = mvv_lva(attacker, victim);

            start += 1;
        }
    }

    fn score_quiet_moves(&mut self, board: &Board, history: &History) {
        for elem in self
            .list
            .iter_mut()
            .skip(self.idx)
            .take(self.limit - self.idx)
        {
            debug_assert!(elem.mv.is_quiet());
            elem.score = history.score(board, elem.mv) as i16;
        }
    }

    pub fn pick<const INCLUDE_QUIETS: bool>(
        &mut self,
        board: &Board,
        history: &History,
        tt_move: Move,
        killer: Move,
    ) -> Option<Move> {
        loop {
            while self.stage_complete() {
                self.advance_stage();

                match self.stage {
                    MoveStage::TT_MOVE => {
                        if tt_move.is_pseudolegal(board) {
                            return Some(tt_move);
                        }
                    }
                    MoveStage::NOISY => {
                        self.gen_moves::<true>(board);
                        self.score_noisy_moves(board);
                    }
                    MoveStage::KILLER => {
                        if killer.is_pseudolegal(board) {
                            return Some(tt_move);
                        }
                    }
                    MoveStage::QUIET => {
                        if INCLUDE_QUIETS {
                            self.gen_moves::<false>(board);
                            self.score_quiet_moves(board, history);
                        }
                    }
                    _ => return None,
                }
            }

            let potential_move = self.next_best_move();
            /*
                TODO: can I make this more efficient?
                tt_move and killer could be null, or they could
                just straight up not apply to the position
                ie. if we are in capture stage we wont see killers

                maybe I could store a list of potential repeats in the picker struct...
            */
            if ![tt_move, killer].contains(&potential_move) {
                return Some(potential_move);
            }
        }
    }

    pub fn simple_pick<const INCLUDE_QUIETS: bool>(&mut self, board: &Board) -> Option<Move> {
        let dummy_hist = History::new();
        self.pick::<INCLUDE_QUIETS>(board, &dummy_hist, Move::NULL, Move::NULL)
    }

    pub fn first_legal_mv(board: &Board) -> Option<Move> {
        let mut generator = Self::new();
        generator.simple_pick::<true>(board)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generates_noisy() {
        use super::*;

        let board = Board::from_fen("1n4K1/P2k2b1/4r1n1/PpPB4/5N2/bRq1r3/3P4/2Q5 w - b6 0 2");
        let mut counts = [0; Piece::CNT as usize];
        let mut promo_count = 0;
        let mut ep_count = 0;

        let mut generator = MovePicker::new();
        while let Some(mv) = generator.simple_pick::<false>(&board) {
            let piece = board.piece_on_sq(mv.from());
            counts[piece.as_index()] += 1;

            if mv.is_promo() {
                promo_count += 1;
            }

            if mv.flag() == Flag::EP {
                ep_count += 1;
            }
        }

        assert_eq!(counts[Piece::PAWN.as_index()], 6);
        assert_eq!(counts[Piece::BISHOP.as_index()], 1);
        assert_eq!(counts[Piece::ROOK.as_index()], 3);
        assert_eq!(counts[Piece::QUEEN.as_index()], 2);
        assert_eq!(counts[Piece::KNIGHT.as_index()], 2);
        assert_eq!(counts[Piece::KING.as_index()], 1);
        assert_eq!(promo_count, 2);
        assert_eq!(ep_count, 2);
    }

    #[test]
    fn generates_quiets() {
        use super::*;

        let board = Board::from_fen(
            "r3k2r/pPppqpb1/bn2pnp1/3PN3/1p2P3/1nN2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0",
        );
        let mut counts = [0; Piece::CNT as usize];
        let mut promo_count = 0;
        let mut castle_count = 0;

        let mut generator = MovePicker::new();
        while let Some(mv) = generator.simple_pick::<true>(&board) {
            if !mv.is_noisy() {
                let piece = board.piece_on_sq(mv.from());
                counts[piece.as_index()] += 1;

                if mv.is_promo() {
                    promo_count += 1;
                }

                if mv.flag() == Flag::KS_CASTLE || mv.flag() == Flag::QS_CASTLE {
                    castle_count += 1;
                }
            }
        }

        assert_eq!(counts[Piece::PAWN.as_index()], 11);
        assert_eq!(counts[Piece::BISHOP.as_index()], 10);
        assert_eq!(counts[Piece::ROOK.as_index()], 5);
        assert_eq!(counts[Piece::QUEEN.as_index()], 7);
        assert_eq!(counts[Piece::KNIGHT.as_index()], 8);
        assert_eq!(counts[Piece::KING.as_index()], 3);
        assert_eq!(promo_count, 6);
        assert_eq!(castle_count, 1);
    }

    #[test]
    fn correct_move_count() {
        use super::*;

        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        let expected_count = 48;
        let mut actual = 0;
        let mut list = vec![];

        let mut g = MovePicker::new();
        while let Some(mv) = g.simple_pick::<true>(&board) {
            actual += 1;
            assert!(!list.contains(&mv), "{} is duplicate", mv.as_string());
            list.push(mv);
        }
        assert_eq!(expected_count, actual);
    }
}
