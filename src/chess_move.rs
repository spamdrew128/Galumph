use crate::board_rep::{Piece, Square};

pub const MAX_MOVECOUNT: usize = u8::MAX as usize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag(u8);

impl Flag {
    pub const NONE: Self = Self(0);
    pub const KS_CASTLE: Self = Self(1);
    pub const QS_CASTLE: Self = Self(2);
    pub const DOUBLE_PUSH: Self = Self(3);
    pub const CAPTURE: Self = Self(4);
    pub const EP: Self = Self(5);
    pub const PROMO: Self = Self(8); // 10xx
    pub const CAPTURE_PROMO: Self = Self(12); // 11xx
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    const TO_BITFIELD: u16 = 0b0000000000111111;
    const FROM_BITFIELD: u16 = 0b0000111111000000;
    const FLAG_BITFIELD: u16 = 0b1111000000000000;
    const PROMO_PIECE_BITFIELD: u16 = 0b0011000000000000;

    const FROM_OFFSET: u8 = 6;
    const FLAG_OFFSET: u8 = 12;
    const PROMO_PIECE_OFFSET: u8 = 12;

    const NULL: Self = Self(0);

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    pub const fn new(to: Square, from: Square, flag: Flag) -> Self {
        Self(to.as_u16() | (from.as_u16() << Self::FROM_OFFSET) | (flag.0 << Self::FLAGS_OFFSET))
    }

    pub const fn new_ks_castle(king_sq: Square) -> Self {
        Self(
            king_sq.right(2).as_u16()
                | (king_sq.as_u16() << Self::FROM_OFFSET)
                | (Flag::KS_CASTLE.0 << Self::FLAG_OFFSET),
        )
    }

    pub const fn new_qs_castle(king_sq: Square) -> Self {
        Self(
            king_sq.left(2).as_u16()
                | (king_sq.as_u16() << Self::FROM_OFFSET)
                | (Flag::QS_CASTLE.0 << Self::FLAG_OFFSET),
        )
    }

    pub const fn to(self) -> Square {
        Square::new((self.0 & Self::TO_BITFIELD) as u8)
    }

    pub const fn from(self) -> Square {
        Square::new(((self.0 & Self::FROM_BITFIELD) >> Self::FROM_OFFSET) as u8)
    }

    pub const fn flag(self) -> Flag {
        Flag((self.0 & Self::FLAG_BITFIELD) >> Self::FLAG_OFFSET)
    }

    pub const fn promo(self) -> Piece {
        Piece::new((self.0 & Self::PROMO_PIECE_BITFIELD) >> Self::PROMO_PIECE_OFFSET)
    }

    pub fn is_capture(self) -> bool {
        const CAPTURE_BITMASK: u8 = 0b0100; // this bit is only set for captures
        let flag_bits = self.flag().0;
        flag_bits & CAPTURE_BITMASK > 0
    }

    pub fn is_noisy(self) -> bool {
        let flag = self.flag();
        // the only promos that are noisy are queen
        (flag >= Flag::CAPTURE && flag <= Flag::Promo) || flag == Flag::CAPTURE_PROMO
    }
}
