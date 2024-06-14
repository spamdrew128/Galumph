use crate::move_generation::{
    attacks,
    board_rep::{Board, Color, Piece},
    chess_move::{Flag, Move},
};

pub const SEE_VALS: [i32; (Piece::CNT + 1) as usize] = [450, 450, 650, 1250, 100, 0, 0];

pub const ASCENDING_PIECE_ORDER: [Piece; Piece::CNT as usize] = [
    Piece::PAWN,
    Piece::KNIGHT,
    Piece::BISHOP,
    Piece::ROOK,
    Piece::QUEEN,
    Piece::KING,
];

impl Board {
    pub fn search_see(&self, mv: Move, threshold: i32) -> bool {
        let attacker = self.piece_on_sq(mv.from());
        let victim = self.piece_on_sq(mv.to());
        self.see(mv, attacker, victim, threshold)
    }

    pub fn see(&self, mv: Move, attacker: Piece, victim: Piece, threshold: i32) -> bool {
        let sq = mv.to();
        let mut color = self.stm;
        let mut next = attacker;

        // occupied bitboard after move is made
        let mut occ = (self.occupied() ^ mv.from().as_bitboard()) | sq.as_bitboard();

        let base = if mv.flag() == Flag::EP {
            // remove the captured pawn from occ if move is EP
            occ ^= sq.row_swap().as_bitboard();

            0 // pawn caputured pawn so base is 0
        } else if mv.is_promo() {
            let promo_pc = mv.promo_piece();
            next = promo_pc;

            // we invest the value of a pawn, but we get our promo piece value, plus whatever we captured
            SEE_VALS[victim.as_index()] + SEE_VALS[promo_pc.as_index()]
                - SEE_VALS[Piece::PAWN.as_index()]
        } else {
            // our new value will be our captured piece value, with our investment (piece) subtracted
            SEE_VALS[victim.as_index()] - SEE_VALS[attacker.as_index()]
        };

        // the see capture chain ends when we get a non-negative score
        let mut score = base - threshold;

        // if we captured a higher value piece than we attacked with,
        // we have positive SEE no matter what
        if score >= 0 {
            return true;
        }

        let rooks = self.pieces[Piece::ROOK.as_index()];
        let bishops = self.pieces[Piece::BISHOP.as_index()];
        let queens = self.pieces[Piece::QUEEN.as_index()];

        let hv_sliders = rooks | queens;
        let d_sliders = bishops | queens;

        let mut all_attackers = (attacks::knight(sq) & self.pieces[Piece::KNIGHT.as_index()])
            | (attacks::king(sq) & self.pieces[Piece::KING.as_index()])
            | (attacks::rook(sq, occ) & hv_sliders)
            | (attacks::bishop(sq, occ) & d_sliders)
            | (attacks::pawn(sq, Color::White) & self.piece_bb(Piece::PAWN, Color::Black))
            | (attacks::pawn(sq, Color::Black) & self.piece_bb(Piece::PAWN, Color::White));

        color = color.flip();
        loop {
            let color_bb = self.all[color.as_index()];
            let our_attackers = all_attackers & color_bb;

            if our_attackers.is_empty() {
                break;
            }

            for piece in ASCENDING_PIECE_ORDER {
                let piece_bb = our_attackers & self.pieces[piece.as_index()];
                if piece_bb.not_empty() {
                    occ ^= piece_bb.lsb_bb();
                    next = piece;
                    break;
                }
            }

            if next == Piece::PAWN || next == Piece::BISHOP || next == Piece::QUEEN {
                all_attackers |= attacks::bishop(sq, occ) & bishops;
            }

            if next == Piece::ROOK || next == Piece::QUEEN {
                all_attackers |= attacks::rook(sq, occ) & rooks;
            }

            all_attackers = occ & all_attackers;
            score = -score - 1 - SEE_VALS[next.as_index()];
            color = color.flip();

            if score >= 0 {
                let our_defenders = all_attackers & self.all[color.as_index()];
                // if the square is still defended, the king can't take and the capture chain ends
                if next == Piece::KING && our_defenders.not_empty() {
                    color = color.flip();
                }
                break;
            }
        }

        color != self.stm
    }
}

#[cfg(test)]
mod tests {
    use crate::move_generation::{
        board_rep::{Board, Piece},
        chess_move::Move,
    };

    #[test]
    fn equal_position_see() {
        let board =
            Board::from_fen("rnbqkb1r/ppp1pppp/5n2/3p4/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3");
        let mv = Move::from_str("c3d5", &board).unwrap();

        assert!(board.see(mv, Piece::KNIGHT, Piece::PAWN, 0));
        assert!(!board.see(mv, Piece::KNIGHT, Piece::PAWN, 1));
    }

    #[test]
    fn ep_xray() {
        let board =
            Board::from_fen("1nbqkb1r/1pp1p3/5p1p/p2n2pP/4p3/P1N2Pr1/1PPP2P1/R1BQKBNR w k g6 0 16");
        let mv = Move::from_str("h5g6", &board).unwrap();

        assert!(board.see(mv, Piece::PAWN, Piece::NONE, 0));
        assert!(!board.see(mv, Piece::PAWN, Piece::NONE, 1));
    }

    #[test]
    fn king_cant_end_chain() {
        let board = Board::from_fen("8/3b4/8/5nk1/8/5R2/K4R2/5R2 w - - 0 1");
        let mv = Move::from_str("f3f5", &board).unwrap();

        assert!(board.see(mv, Piece::ROOK, Piece::KNIGHT, 0));
    }
}
