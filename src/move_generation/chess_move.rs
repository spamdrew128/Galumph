use crate::move_generation::{
    attacks,
    board_rep::{Board, Color, Piece, Square},
};

use super::board_rep::Bitboard;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag(u16);

impl Flag {
    pub const NONE: Self = Self(0);
    pub const KS_CASTLE: Self = Self(1);
    pub const QS_CASTLE: Self = Self(2);
    pub const DOUBLE_PUSH: Self = Self(3);
    pub const KNIGHT_PROMO: Self = Self(4);
    pub const BISHOP_PROMO: Self = Self(5);
    pub const ROOK_PROMO: Self = Self(6);
    pub const QUEEN_PROMO: Self = Self(7);
    pub const CAPTURE: Self = Self(8);
    pub const EP: Self = Self(9);
    // skip entry for promo piece purposes
    pub const QUEEN_CAPTURE_PROMO: Self = Self(11);
    pub const KNIGHT_CAPTURE_PROMO: Self = Self(12);
    pub const BISHOP_CAPTURE_PROMO: Self = Self(13);
    pub const ROOK_CAPTURE_PROMO: Self = Self(14);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    const FROM_BITFIELD: u16 = 0b0000000000111111;
    const TO_BITFIELD: u16 = 0b0000111111000000;
    const FLAG_BITFIELD: u16 = 0b1111000000000000;
    const PROMO_PIECE_BITFIELD: u16 = 0b0011000000000000;

    const FROM_OFFSET: u8 = 0;
    const TO_OFFSET: u8 = 6;
    const FLAG_OFFSET: u8 = 12;
    const PROMO_PIECE_OFFSET: u8 = 12;

    pub const NULL: Self = Self(0);

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    pub const fn new(to: Square, from: Square, flag: Flag) -> Self {
        Self(
            (from.as_u16() << Self::FROM_OFFSET)
                | (to.as_u16() << Self::TO_OFFSET)
                | (flag.0 << Self::FLAG_OFFSET),
        )
    }

    pub const fn new_ks_castle(king_sq: Square) -> Self {
        Self(
            (king_sq.as_u16() << Self::FROM_OFFSET)
                | (king_sq.right(2).as_u16() << Self::TO_OFFSET)
                | (Flag::KS_CASTLE.0 << Self::FLAG_OFFSET),
        )
    }

    pub const fn new_qs_castle(king_sq: Square) -> Self {
        Self(
            (king_sq.as_u16() << Self::FROM_OFFSET)
                | (king_sq.left(2).as_u16() << Self::TO_OFFSET)
                | (Flag::QS_CASTLE.0 << Self::FLAG_OFFSET),
        )
    }

    pub const fn from(self) -> Square {
        let sq_bits = (self.0 & Self::FROM_BITFIELD) >> Self::FROM_OFFSET;
        Square::new(sq_bits as u8)
    }

    pub const fn to(self) -> Square {
        let sq_bits = (self.0 & Self::TO_BITFIELD) >> Self::TO_OFFSET;
        Square::new(sq_bits as u8)
    }

    pub const fn flag(self) -> Flag {
        let flag_bits = (self.0 & Self::FLAG_BITFIELD) >> Self::FLAG_OFFSET;
        Flag(flag_bits)
    }

    pub fn is_capture(self) -> bool {
        self.flag() >= Flag::CAPTURE
    }

    pub fn is_noisy(self) -> bool {
        self.flag() >= Flag::QUEEN_PROMO && self.flag() <= Flag::QUEEN_CAPTURE_PROMO
    }

    pub fn is_promo(self) -> bool {
        // very inefficient so probably avoid where speed matters :p
        self.flag() >= Flag::QUEEN_CAPTURE_PROMO
            || (self.flag() >= Flag::KNIGHT_PROMO && self.flag() <= Flag::QUEEN_PROMO)
    }

    pub fn promo_piece(self) -> Piece {
        let piece_bits = (self.0 & Self::PROMO_PIECE_BITFIELD) >> Self::PROMO_PIECE_OFFSET;
        Piece::new(piece_bits as u8)
    }

    pub fn as_string(self) -> String {
        if self.is_null() {
            return "NULL".to_owned();
        }

        let mut move_str = String::new();
        move_str.push_str(self.from().as_string().as_str());
        move_str.push_str(self.to().as_string().as_str());

        if self.is_promo() {
            move_str.push(self.promo_piece().as_char(Color::Black));
        }

        move_str
    }

    pub fn from_str(mv_str: &str, board: &Board) -> Option<Self> {
        if mv_str.len() > 5 || mv_str.len() < 4 {
            return None;
        }

        let mut chars = mv_str.chars();
        let from_str = format!("{}{}", chars.next().unwrap(), chars.next().unwrap());
        let to_str = format!("{}{}", chars.next().unwrap(), chars.next().unwrap());
        let promo = chars.next();

        let from = Square::from_string(from_str.as_str());
        let to = Square::from_string(to_str.as_str());
        if from.is_none() || to.is_none() {
            return None;
        }

        let from = from.unwrap();
        let to = to.unwrap();
        let piece = board.piece_on_sq(from);
        let captured_piece = board.piece_on_sq(to);

        let promo_flags = [
            Flag::KNIGHT_PROMO,
            Flag::BISHOP_PROMO,
            Flag::ROOK_PROMO,
            Flag::QUEEN_PROMO,
        ];
        let cap_promo_flags = [
            Flag::KNIGHT_CAPTURE_PROMO,
            Flag::BISHOP_CAPTURE_PROMO,
            Flag::ROOK_CAPTURE_PROMO,
            Flag::QUEEN_CAPTURE_PROMO,
        ];

        if piece == Piece::KING && (!attacks::king(from).overlaps(to.as_bitboard())) {
            if to.file() >= from.file() {
                return Some(Self::new_ks_castle(from));
            }
            if to.file() <= from.file() {
                return Some(Self::new_qs_castle(from));
            }
        }

        if board.promotable_pawns().overlaps(from.as_bitboard()) {
            let promo_type = Piece::from_char(promo.unwrap()).unwrap();
            let flag = if captured_piece == Piece::NONE {
                promo_flags[promo_type.as_index()]
            } else {
                cap_promo_flags[promo_type.as_index()]
            };
            return Some(Self::new(to, from, flag));
        }

        if piece == Piece::PAWN {
            if let Some(ep_sq) = board.ep_sq {
                if ep_sq == to {
                    return Some(Self::new(to, from, Flag::EP));
                }
            }

            if from == to.double_push_sq() {
                return Some(Self::new(to, from, Flag::DOUBLE_PUSH));
            }
        }

        if captured_piece == Piece::NONE {
            Some(Self::new(to, from, Flag::NONE))
        } else {
            Some(Self::new(to, from, Flag::CAPTURE))
        }
    }

    pub fn is_pseudolegal(self, board: &Board) -> bool {
        // we can't play null moves!
        if self.is_null() {
            return false;
        }

        let to = self.to();
        let to_bb = to.as_bitboard();
        let from = self.from();
        let from_bb = from.as_bitboard();
        let us = board.us();
        let them = board.them();
        let occupied = board.occupied();
        let empty = !occupied;
        let flag = self.flag();

        // make sure to move a piece that is our color, and non-empty
        if !from_bb.overlaps(us) {
            return false;
        }

        // we actually need to capture an enemy piece if the move is a capture (and not en passant)
        if self.is_capture() && flag != Flag::EP && !to_bb.overlaps(them) {
            return false;
        }

        // if non-capture, we need to land on an unoccupied square
        if !self.is_capture() && to_bb.overlaps(occupied) {
            return false;
        }

        let piece = board.piece_on_sq(from);
        let color = board.stm;
        match flag {
            Flag::NONE | Flag::CAPTURE => {
                let moves_bb = match piece {
                    Piece::KNIGHT => attacks::knight(from),
                    Piece::KING => attacks::king(from),
                    Piece::BISHOP => attacks::bishop(from, occupied),
                    Piece::ROOK => attacks::rook(from, occupied),
                    Piece::QUEEN => attacks::queen(from, occupied),
                    _ => {
                        // assume pawn
                        let pawn: Bitboard = from_bb.without(board.promotable_pawns());
                        if flag == Flag::NONE {
                            attacks::pawn_single_push(pawn, empty, color)
                        } else {
                            attacks::pawn(from, color)
                        }
                    }
                };

                to_bb.overlaps(moves_bb)
            }
            Flag::DOUBLE_PUSH => {
                let single_push = attacks::pawn_single_push(from_bb, empty, color);
                let double_push = attacks::pawn_double_push(single_push, empty, color);
                (piece == Piece::PAWN) && to_bb.overlaps(double_push)
            }
            Flag::KS_CASTLE => board.can_ks_castle(),
            Flag::QS_CASTLE => board.can_qs_castle(),
            Flag::EP => board.ep_sq.map_or(false, |ep_sq| {
                (piece == Piece::PAWN)
                    && (ep_sq == to)
                    && attacks::pawn(from, color).overlaps(ep_sq.as_bitboard())
            }),
            _ => {
                // assume promotion
                let pawn: Bitboard = from_bb & board.promotable_pawns();
                let move_bb = if self.is_capture() {
                    attacks::pawn(from, color)
                } else {
                    attacks::pawn_single_push(pawn, empty, color)
                };

                to_bb.overlaps(move_bb)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::move_generation::{
        board_rep::{Board, Piece, Square},
        movegen::MovePicker,
        perft::{test_postions, PerftTest},
    };

    use super::{Flag, Move};

    #[test]
    fn basic_test() {
        let to = Square::new(0b10110);
        let from = Square::new(0b101);
        let flag = Flag::QUEEN_CAPTURE_PROMO;

        let mv = Move::new(to, from, flag);
        assert_eq!(to, mv.to());
        assert_eq!(from, mv.from());
        assert_eq!(flag, mv.flag());
        assert_eq!(Piece::QUEEN, mv.promo_piece());

        assert!(mv.is_capture());
        assert!(mv.is_noisy());
    }

    #[test]
    fn is_pseudolegal_false_positives() {
        let positions: Vec<PerftTest> = test_postions();

        for pos1 in &positions {
            let board_1 = Board::from_fen(pos1.fen);
            let mut b1_generator = MovePicker::new();
            let mut actual_pseudos = vec![];
            while let Some(mv) = b1_generator.simple_pick::<true>(&board_1) {
                actual_pseudos.push(mv);
            }

            for pos2 in &positions {
                let mut b2_generator = MovePicker::new();
                let board_2 = Board::from_fen(pos2.fen);
                while let Some(mv) = b2_generator.simple_pick::<true>(&board_2) {
                    let expected = actual_pseudos.contains(&mv);
                    let actual = mv.is_pseudolegal(&board_1);

                    assert_eq!(
                        expected,
                        actual,
                        "\nFen_1: {}\nFen_2: {}\nMove: {}\nFlag {}",
                        board_1.as_fen(),
                        board_2.as_fen(),
                        mv.as_string(),
                        mv.flag().0,
                    );
                }
            }
        }
    }
}
