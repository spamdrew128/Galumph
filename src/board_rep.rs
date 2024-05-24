use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

use crate::tuple_constants_enum;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub const CNT: u8 = 64;

    #[rustfmt::skip]
    tuple_constants_enum!(Self,
        A8, B8, C8, D8, E8, F8, G8, H8,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A1, B1, C1, D1, E1, F1, G1, H1
    );

    pub const fn new(data: u8) -> Self {
        Self(data)
    }

    pub const fn as_bitboard(self) -> Bitboard {
        Bitboard::new(1 << self.0)
    }

    pub const fn as_u16(self) -> u16 {
        self.0 as u16
    }

    pub const fn as_index(self) -> usize {
        self.0 as usize
    }

    pub const fn left(self, count: u8) -> Self {
        Self(self.0 - count)
    }

    pub const fn right(self, count: u8) -> Self {
        Self(self.0 + count)
    }

    pub const fn mirror(self) -> Self {
        Self(self.0 ^ 0b111000)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}
impl Direction {
    pub const LIST: [Direction; 8] = [
        Direction::N,
        Direction::NE,
        Direction::E,
        Direction::SE,
        Direction::S,
        Direction::SW,
        Direction::W,
        Direction::NW,
    ];
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const EMPTY: Self = Self::new(0);
    pub const A_FILE: Self = Self::new(0x0101010101010101);
    pub const H_FILE: Self = Self::new(0x8080808080808080);

    pub const fn new(data: u64) -> Self {
        Self(data)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn shift(self, dir: Direction, shift: u8) -> Self {
        match dir {
            Direction::N => Self(self.0 >> (8 * shift)),
            Direction::S => Self(self.0 << (8 * shift)),
            _ => {
                let mut i = 0;
                let mut data = self.0;
                while i < shift {
                    data = match dir {
                        Direction::NE => (data & !Self::H_FILE.0) >> 7,
                        Direction::E => (data & !Self::H_FILE.0) << 1,
                        Direction::SE => (data & !Self::H_FILE.0) << 9,
                        Direction::SW => (data & !Self::A_FILE.0) << 7,
                        Direction::W => (data & !Self::A_FILE.0) >> 1,
                        Direction::NW => (data & !Self::A_FILE.0) >> 9,
                        _ => panic!("Invalid direction"),
                    };
                    i += 1;
                }
                Self(data)
            }
        }
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn not_empty(self) -> bool {
        self.0 != 0
    }

    pub const fn or(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    pub const fn and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    pub const fn not(self) -> Self {
        Self(!self.0)
    }

    pub const fn overlaps(self, rhs: Self) -> bool {
        self.and(rhs).not_empty()
    }

    pub const fn popcount(self) -> u8 {
        self.0.count_ones() as u8
    }

    pub fn print(self) {
        fn fen_index_as_bitboard(i: u8) -> Bitboard {
            let row = i / 8;
            let col = i % 8;
            Bitboard::new(1 << (row * 8 + col))
        }

        for i in 0..Square::CNT {
            let bitset = fen_index_as_bitboard(i);
            if bitset.overlaps(self) {
                print!("X ");
            } else {
                print!(". ");
            }

            if (i + 1) % 8 == 0 {
                println!();
            }
        }
        println!();
    }
}

impl Shl<u8> for Bitboard {
    type Output = Self;

    fn shl(self, shift: u8) -> Self::Output {
        Self(self.0 << shift)
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;

    fn shr(self, shift: u8) -> Self::Output {
        Self(self.0 >> shift)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Piece(u8);

impl Piece {
    pub const CNT: u8 = 6;

    // QUEEN MUST BE TOP FOR NOISY DETECTION
    #[rustfmt::skip]
    tuple_constants_enum!(Self,
        KNIGHT,
        BISHOP,
        ROOK,
        QUEEN,
        PAWN,
        KING,
        NONE
    );

    pub const LIST: [Self; Self::CNT as usize] = [
        Self::KNIGHT,
        Self::BISHOP,
        Self::ROOK,
        Self::QUEEN,
        Self::PAWN,
        Self::KING,
    ];

    pub const fn new(data: u8) -> Self {
        Self(data)
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Color {
    #[default]
    White,
    Black,
}

impl Color {
    pub const CNT: u8 = 2;
    pub const LIST: [Self; Self::CNT as usize] = [Self::White, Self::Black];

    pub const fn flip(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
    pub const fn as_index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameState {
    pub all: [Bitboard; Color::CNT as usize],
    pub pieces: [Bitboard; Piece::CNT as usize],
    // pub color_to_move: Color,
    // pub ep_sq: Option<Square>,
    // pub castle_rights: CastleRights,
    // pub halfmoves: u16,
}

#[cfg(test)]
mod tests {
    use crate::board_rep::{Direction, Square};

    #[test]
    fn shifting_test() {
        let bb = Square::B2.as_bitboard();
        bb.print();
        println!("---------------------");

        for dir in Direction::LIST {
            bb.shift(dir, 1).print();
        }
    }
}
