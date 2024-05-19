use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

pub const SQ_CNT: u8 = 64;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Bitboard(u64);

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

struct Direction;
impl Direction {
    const N: u8 = 0;
    const NE: u8 = 1;
    const E: u8 = 2;
    const SE: u8 = 3;
    const S: u8 = 4;
    const SW: u8 = 5;
    const W: u8 = 6;
    const NW: u8 = 7;
}

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

    const fn shift<const DIR: u8>(self, shift: u8) -> Self {
        match DIR {
            Direction::N => Self(self.0 << (8 * shift)),
            Direction::S => Self(self.0 >> (8 * shift)),
            _ => {
                let mut i = 0;
                let mut data = self.0;
                while i < shift {
                    data = match DIR {
                        Direction::NE => (data & !Self::H_FILE.0) << 9,
                        Direction::E => (data & !Self::H_FILE.0) << 1,
                        Direction::SE => (data & !Self::H_FILE.0) >> 7,
                        Direction::SW => (data & !Self::A_FILE.0) >> 9,
                        Direction::W => (data & !Self::A_FILE.0) >> 1,
                        Direction::NW => (data & !Self::A_FILE.0) << 7,
                        _ => panic!("Invalid direction"),
                    };
                    i += 1;
                }
                Self(data)
            }
        }
    }

    pub fn overlaps(self, rhs: Self) -> bool {
        (self & rhs) != Bitboard::EMPTY
    }

    pub const fn popcount(self) -> u8 {
        self.0.count_ones() as u8
    }

    pub fn print(self) {
        for i in 0..SQ_CNT {
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

fn fen_index_as_bitboard(i: u8) -> Bitboard {
    let row = 7 - (i / 8);
    let col = i % 8;
    Bitboard::new(1 << (row * 8 + col))
}
