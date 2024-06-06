use crate::{
    bitloop,
    move_generation::{
        attacks,
        board_rep::{Bitboard, Board, Piece, Square},
        chess_move::{Flag, Move},
    },
};

macro_rules! into_moves {
    (|$from:ident| $piece_bb:ident, |$to:ident| $moves_bb:expr, $add_move:expr) => {{
        bitloop!(|$from| $piece_bb, {
            let moves: Bitboard = $moves_bb;
            bitloop!(|$to| moves, { $add_move });
        });
    }};
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

pub struct MovePicker {
    list: [ScoredMove; Self::SIZE],
    idx: usize,
    len: usize,
}

impl MovePicker {
    const SIZE: usize = u8::MAX as usize;

    fn add(&mut self, mv: Move) {
        self.list[self.len].mv = mv;
        self.len += 1;
    }

    fn take(&mut self) -> Move {
        let mv = self.list[self.idx].mv;
        self.idx += 1;
        mv
    }

    pub fn new(board: &Board) -> Self {
        let mut res = Self {
            list: [ScoredMove::EMPTY; Self::SIZE],
            idx: 0,
            len: 0,
        };
        res.gen_moves::<true>(board);
        res.gen_moves::<false>(board);

        res
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

    pub fn pick(&mut self) -> Option<Move> {
        if self.idx < self.len {
            Some(self.take())
        } else {
            None
        }
    }
}
