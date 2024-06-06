use crate::move_generation::{
    attacks,
    board_rep::{Board, Color, Piece, Square},
};

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

    pub fn is_pseudolegal(self, _board: &Board) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::move_generation::board_rep::{Piece, Square};

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
}
