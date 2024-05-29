macro_rules! tuple_constants_enum {
    ($t:ty, $($n:ident),*) => {
        tuple_constants_enum!($t, 0, $($n),*);
    };
    ($t:ty, $val:expr, $name:ident) => {
        pub const $name: $t = <$t>::new($val);
    };
    ($t:ty, $val:expr, $name:ident, $($n:ident),*) => {
        pub const $name: $t = <$t>::new($val);
        tuple_constants_enum!($t, $val + 1, $($n),*);
    };
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub const CNT: u8 = 64;
    pub const RANK_CNT: u8 = 8;
    pub const COL_CNT: u8 = 8;

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

    pub const fn rank(self) -> u8 {
        7 - self.0 / 8
    }

    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    pub const fn left(self, count: u8) -> Self {
        Self(self.0 - count)
    }

    pub const fn right(self, count: u8) -> Self {
        Self(self.0 + count)
    }

    pub const fn row_swap(self) -> Self {
        // even rows become odd, odd rows become even
        Self::new(self.0 ^ 0b1000)
    }

    pub const fn double_push_sq(self) -> Self {
        Self::new(self.0 ^ 0b10000)
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

    pub const RANK_1: Self = Self::new(0xff00000000000000);
    pub const RANK_2: Self = Self::new(0x00ff000000000000);
    pub const RANK_4: Self = Self::new(0x000000ff00000000);
    pub const RANK_5: Self = Self::new(0x00000000ff000000);
    pub const RANK_7: Self = Self::new(0x000000000000ff00);
    pub const RANK_8: Self = Self::new(0x00000000000000ff);

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

    const fn lsb(self) -> Square {
        Square::new(self.0.trailing_zeros() as u8)
    }

    fn reset_lsb(&mut self) {
        self.0 = self.0 & (self.0 - 1);
    }

    pub fn pop_lsb(&mut self) -> Square {
        debug_assert!(self.not_empty());
        let sq = self.lsb();
        self.reset_lsb();
        sq
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

#[cfg(test)]
mod tests {
    use crate::{
        movegen::board_rep::{Board, Direction, Square},
        movegen::perft,
    };

    #[test]
    fn shifting_test() {
        let bb = Square::B2.as_bitboard();
        bb.print();
        println!("---------------------");

        for dir in Direction::LIST {
            bb.shift(dir, 1).print();
        }
    }

    #[test]
    fn fen_test() {
        let test_postions = perft::test_postions();
        for pos in test_postions {
            let fen = pos.fen;
            assert_eq!(Board::from_fen(fen).as_fen(), fen);
        }
    }
}
