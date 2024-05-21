use crate::{
    board_rep::{Bitboard, Color, Direction, Square},
    magic::MagicHashTable,
};

macro_rules! init_attacks {
    (|$sq_bb:ident| $body:expr) => {{
        let mut res = [Bitboard::EMPTY; Square::CNT as usize];
        let mut i = 0;
        while i < Square::CNT {
            let $sq_bb = Square::new(i).as_bitboard();
            res[i as usize] = $body;
            i += 1;
        }
        res
    }};
}

const PAWN_ATTACKS: [[Bitboard; Square::CNT as usize]; Color::CNT as usize] = [
    init_attacks!(|sq_bb| {
        // Init white pawn attacks
        sq_bb
            .shift(Direction::NE, 1)
            .or(sq_bb.shift(Direction::NW, 1))
    }),
    init_attacks!(|sq_bb| {
        // Init black pawn attacks
        sq_bb
            .shift(Direction::SE, 1)
            .or(sq_bb.shift(Direction::SW, 1))
    }),
];

const KNIGHT_ATTACKS: [Bitboard; Square::CNT as usize] = init_attacks!(|sq_bb| {
    let vert = sq_bb
        .shift(Direction::N, 2)
        .or(sq_bb.shift(Direction::S, 2));
    let horiz = sq_bb
        .shift(Direction::E, 2)
        .or(sq_bb.shift(Direction::W, 2));
    vert.shift(Direction::E, 1)
        .or(vert.shift(Direction::W, 1))
        .or(horiz.shift(Direction::N, 1))
        .or(horiz.shift(Direction::S, 1))
});

const KING_ATTACKS: [Bitboard; Square::CNT as usize] = init_attacks!(|sq_bb| {
    sq_bb
        .shift(Direction::N, 1)
        .or(sq_bb.shift(Direction::NE, 1))
        .or(sq_bb.shift(Direction::E, 1))
        .or(sq_bb.shift(Direction::SE, 1))
        .or(sq_bb.shift(Direction::S, 1))
        .or(sq_bb.shift(Direction::SW, 1))
        .or(sq_bb.shift(Direction::W, 1))
        .or(sq_bb.shift(Direction::NW, 1))
});

// #[allow(long_running_const_eval)]
// const MAGIC_HASH_TABLE: MagicHashTable = MagicHashTable::generate();
