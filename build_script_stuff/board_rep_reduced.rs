use bytemuck::Zeroable;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub const CNT: u8 = 64;

    pub const fn new(data: u8) -> Self {
        Self(data)
    }

    pub const fn as_bitboard(self) -> Bitboard {
        Bitboard::new(1 << self.0)
    }

    pub const fn as_index(self) -> usize {
        self.0 as usize
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Zeroable)]
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
}

#[cfg(test)]
mod tests {
    use crate::{
        movegen::board_rep::{Board, Direction, Square},
        movegen::perft,
    };

    #[test]
    fn fen_test() {
        let test_postions = perft::test_postions();
        for pos in test_postions {
            let fen = pos.fen;
            assert_eq!(Board::from_fen(fen).as_fen(), fen);
        }
    }
}
