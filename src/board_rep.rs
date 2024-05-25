use crate::{bb_from_squares, tuple_constants_enum};
use std::{
    char,
    ops::{BitAnd, BitOr, BitOrAssign, Not, Shl, Shr},
};

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

    pub const fn left(self, count: u8) -> Self {
        Self(self.0 - count)
    }

    pub const fn right(self, count: u8) -> Self {
        Self(self.0 + count)
    }

    pub const fn mirror(self) -> Self {
        Self(self.0 ^ 0b111000)
    }

    fn from_string(s: &str) -> Option<Self> {
        if s.len() != 2 {
            return None;
        }

        let mut chars = s.chars();
        let file_char = chars.next().unwrap().to_ascii_lowercase();
        let rank_char = chars.next().unwrap();

        let file_num: u8 = (file_char as u8) - ('a' as u8);
        let rank_num: u8 = 7 - ((rank_char as u8) - ('1' as u8));
        let pos = rank_num * 8 + file_num;

        if pos >= Square::CNT {
            return None;
        }

        Some(Square(pos))
    }

    fn as_string(self) -> String {
        let mut res = String::new();

        let file_num = self.0 % 8;
        let rank_num = 7 - (self.0 / 8);

        let file_char = (file_num + ('a' as u8)) as char;
        let rank_char = (rank_num + ('1' as u8)) as char;

        res.push(file_char);
        res.push(rank_char);
        res
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

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
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

    pub const fn as_index(self) -> usize {
        self.0 as usize
    }

    fn from_char(ch: char) -> Option<Self> {
        match ch {
            'n' | 'N' => Some(Self::KNIGHT),
            'b' | 'B' => Some(Self::BISHOP),
            'r' | 'R' => Some(Self::ROOK),
            'q' | 'Q' => Some(Self::QUEEN),
            'p' | 'P' => Some(Self::PAWN),
            'k' | 'K' => Some(Self::KING),
            _ => None,
        }
    }

    fn as_char(self, color: Color) -> char {
        let ch = match self {
            Self::KNIGHT => 'N',
            Self::BISHOP => 'B',
            Self::ROOK => 'R',
            Self::QUEEN => 'Q',
            Self::PAWN => 'P',
            Self::KING => 'K',
            _ => panic!("INVALID PIECE"),
        };

        if color == Color::Black {
            ch.to_ascii_lowercase()
        } else {
            ch
        }
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

    fn from_char(ch: char) -> Option<Self> {
        match ch {
            'w' | 'W' => Some(Self::White),
            'b' | 'B' => Some(Self::Black),
            _ => None,
        }
    }

    fn as_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct CastleRights(u8);

impl CastleRights {
    const W_KS: u8 = 0b0001;
    const W_QS: u8 = 0b0010;
    const B_KS: u8 = 0b0100;
    const B_QS: u8 = 0b1000;

    const KS_THRU: [Bitboard; Color::CNT as usize] = [bb_from_squares!(F1), bb_from_squares!(F8)];
    const QS_THRU: [Bitboard; Color::CNT as usize] =
        [bb_from_squares!(C1, D1), bb_from_squares!(C8, D8)];
    const KS_OCC: [Bitboard; Color::CNT as usize] =
        [bb_from_squares!(F1, G1), bb_from_squares!(F8, G8)];
    const QS_OCC: [Bitboard; Color::CNT as usize] =
        [bb_from_squares!(B1, C1, D1), bb_from_squares!(B8, C8, D8)];

    const UPDATE_MASKS: [u8; Square::CNT as usize] = {
        let mut table = [0b1111; Square::CNT as usize];
        table[Square::A1.as_index()] ^= Self::W_QS;
        table[Square::A8.as_index()] ^= Self::B_QS;
        table[Square::H1.as_index()] ^= Self::W_KS;
        table[Square::H8.as_index()] ^= Self::B_KS;
        table[Square::E1.as_index()] ^= Self::W_KS | Self::W_QS;
        table[Square::E8.as_index()] ^= Self::B_KS | Self::B_QS;
        table
    };

    fn new() -> Self {
        Self(0)
    }

    fn from_str(s: &str) -> Self {
        let mut bits = 0;
        for ch in s.chars() {
            match ch {
                'K' => bits |= Self::W_KS,
                'Q' => bits |= Self::W_QS,
                'k' => bits |= Self::B_KS,
                'q' => bits |= Self::B_QS,
                _ => {}
            }
        }
        Self(bits)
    }

    fn as_string(self) -> String {
        let mut res = String::new();
        let masks = [Self::W_KS, Self::W_QS, Self::B_KS, Self::B_QS];
        let chars = ['K', 'Q', 'k', 'q'];

        for (&mask, &ch) in masks.iter().zip(chars.iter()) {
            if self.0 & mask > 0 {
                res.push(ch);
            }
        }

        if res.is_empty() {
            res.push('-');
        }

        res
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameState {
    pub stm: Color,
    pub all: [Bitboard; Color::CNT as usize],
    pub pieces: [Bitboard; Piece::CNT as usize],
    pub ep_sq: Option<Square>,
    pub castle_rights: CastleRights,
    pub halfmoves: u16,
}

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";

impl GameState {
    fn new() -> Self {
        Self {
            stm: Color::White,
            all: [Bitboard::EMPTY; Color::CNT as usize],
            pieces: [Bitboard::EMPTY; Piece::CNT as usize],
            ep_sq: None,
            castle_rights: CastleRights::new(),
            halfmoves: 0,
        }
    }

    fn piece_on_sq(&self, sq: Square) -> Piece {
        let bitset = sq.as_bitboard();
        for piece in Piece::LIST {
            if bitset.overlaps(self.pieces[piece.as_index()]) {
                return piece;
            }
        }
        Piece::NONE
    }

    fn from_fen(fen: &str) -> Self {
        let mut state = Self::new();
        let mut split = fen.split_whitespace();

        let piece_grid = split.next().unwrap();
        let stm = split.next().unwrap().chars().next().unwrap();
        let castling = split.next().unwrap();
        let ep = split.next().unwrap();
        let halfmoves = split.next().unwrap();

        let rows = piece_grid.split('/');
        let mut i = 0;
        for row_str in rows {
            let chars: Vec<char> = row_str.chars().collect();

            for ch in chars {
                let bitset = Square::new(i).as_bitboard();

                if let Some(piece) = Piece::from_char(ch) {
                    state.all[ch.is_lowercase() as usize] |= bitset;
                    state.pieces[piece.as_index()] |= bitset;
                    i += 1;
                } else {
                    i += ch.to_digit(10).unwrap() as u8;
                }
            }
        }
        assert_eq!(i, Square::CNT);

        state.stm = Color::from_char(stm).unwrap();
        state.castle_rights = CastleRights::from_str(castling);
        state.ep_sq = Square::from_string(ep);
        state.halfmoves = halfmoves.parse::<u16>().unwrap();

        state
    }

    fn as_fen(&self) -> String {
        let mut res = String::new();

        let mut sq_num = 0;
        for _ in 0..Square::RANK_CNT {
            let mut empty = 0;
            let mut rank_string = String::new();
            for _ in 0..Square::COL_CNT {
                let sq = Square(sq_num);
                let piece = self.piece_on_sq(sq);

                if piece == Piece::NONE {
                    empty += 1
                } else {
                    if empty > 0 {
                        rank_string.push(char::from_digit(empty, 10).unwrap());
                        empty = 0;
                    }

                    for color in Color::LIST {
                        if sq.as_bitboard().overlaps(self.all[color.as_index()]) {
                            rank_string.push(piece.as_char(color));
                            break;
                        }
                    }
                }

                sq_num += 1
            }

            if empty > 0 {
                rank_string.push(char::from_digit(empty, 10).unwrap());
            }

            if sq_num < Square::CNT {
                rank_string.push('/');
            }

            res.push_str(rank_string.as_str());
        }

        res.push(' ');
        res.push(self.stm.as_char());
        res.push(' ');
        res.push_str(self.castle_rights.as_string().as_str());
        res.push(' ');
        if let Some(ep) = self.ep_sq {
            res.push_str(ep.as_string().as_str());
        } else {
            res.push('-');
        }
        res.push(' ');
        res.push_str(self.halfmoves.to_string().as_str());
        res.push(' ');
        res.push('1');

        res
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board_rep::{Direction, GameState, Square},
        perft,
    };

    use super::Bitboard;

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
            assert_eq!(GameState::from_fen(fen).as_fen(), fen);
        }
    }
}
