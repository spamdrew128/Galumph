use crate::{
    bitloop,
    move_generation::board_rep::{Board, CastleRights, Color, Piece, Square},
};

const NUM_CASTLING_CONFIGURATIONS: usize = 16;
const NUM_FILES: usize = 8;

#[repr(C)]
struct ZobristKeys {
    pieces: [[[u64; Square::CNT as usize]; Piece::CNT as usize]; Color::CNT as usize],
    castling: [u64; NUM_CASTLING_CONFIGURATIONS],
    ep_file: [u64; NUM_FILES],
    black_to_move: u64,
}

const KEYS: ZobristKeys = unsafe {
    std::mem::transmute(*include_bytes!(concat!(
        env!("OUT_DIR"),
        "/zobrist_init.bin"
    )))
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ZobristHash(u64);

impl ZobristHash {
    pub const EMPTY: Self = Self(0);

    pub fn hash_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        self.0 ^= KEYS.pieces[color.as_index()][piece.as_index()][sq.as_index()];
    }

    pub fn hash_castling(&mut self, castle_rights: CastleRights) {
        self.0 ^= KEYS.castling[castle_rights.as_index()];
    }

    pub fn hash_ep(&mut self, ep_sq: Square) {
        self.0 ^= KEYS.ep_file[ep_sq.file() as usize];
    }

    pub const fn combine(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    pub fn complete(board: &Board) -> Self {
        let mut hash = if board.stm == Color::Black {
            Self(KEYS.black_to_move)
        } else {
            Self(0)
        };

        for color in Color::LIST {
            for piece in Piece::LIST {
                bitloop!(|sq| board.piece_bb(piece, color), {
                    hash.hash_piece(color, piece, sq);
                });
            }
        }

        hash.hash_castling(board.castle_rights);

        if let Some(ep_sq) = board.ep_sq {
            hash.hash_ep(ep_sq);
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use crate::move_generation::chess_move::Move;

    #[test]
    #[rustfmt::skip]
    fn zobrist_transpositions() {
        // credit to Cozy Chess for this test
        // https://github.com/analog-hors/cozy-chess/blob/master/cozy-chess/src/board/zobrist.rs#L191 
        use super::*;
        const MOVES: &[[[&str; 4]; 2]] = &[
            [["e2c4", "h8f8", "d2h6", "b4b3"], ["e2c4", "b4b3", "d2h6", "h8f8"]],
            [["c3a4", "f6g8", "e1d1", "a8c8"], ["c3a4", "a8c8", "e1d1", "f6g8"]],
            [["h1g1", "f6g4", "d2h6", "b4b3"], ["h1g1", "b4b3", "d2h6", "f6g4"]],
            [["a1c1", "c7c5", "c3a4", "a6e2"], ["c3a4", "c7c5", "a1c1", "a6e2"]],
            [["e2c4", "h8h5", "f3f5", "e7d8"], ["f3f5", "h8h5", "e2c4", "e7d8"]],
            [["d5d6", "e8h8", "f3f6", "a6c4"], ["f3f6", "a6c4", "d5d6", "e8h8"]],
            [["f3e3", "e8h8", "a2a4", "a8c8"], ["a2a4", "a8c8", "f3e3", "e8h8"]],
            [["e1d1", "f6d5", "b2b3", "a8c8"], ["e1d1", "a8c8", "b2b3", "f6d5"]],
            [["e1d1", "e8f8", "e5c6", "h8h5"], ["e1d1", "h8h5", "e5c6", "e8f8"]],
            [["e2d3", "c7c6", "g2g4", "h8h6"], ["e2d3", "h8h6", "g2g4", "c7c6"]],
            [["f3h5", "f6h7", "c3b1", "g7f6"], ["c3b1", "f6h7", "f3h5", "g7f6"]],
            [["e2d3", "g6g5", "d2f4", "b6d5"], ["d2f4", "g6g5", "e2d3", "b6d5"]],
            [["a2a3", "h8h5", "c3b1", "a8d8"], ["a2a3", "a8d8", "c3b1", "h8h5"]],
            [["a2a4", "e8h8", "e1h1", "e7d8"], ["e1h1", "e8h8", "a2a4", "e7d8"]],
            [["b2b3", "e8f8", "g2g3", "a6b7"], ["b2b3", "a6b7", "g2g3", "e8f8"]],
            [["e5g4", "e8d8", "d2e3", "a6d3"], ["d2e3", "a6d3", "e5g4", "e8d8"]],
            [["g2h3", "e7d8", "e5g4", "b6c8"], ["e5g4", "b6c8", "g2h3", "e7d8"]],
            [["e5d3", "a6b7", "g2g3", "h8h6"], ["e5d3", "h8h6", "g2g3", "a6b7"]],
            [["e5g4", "h8h5", "f3f5", "e6f5"], ["f3f5", "e6f5", "e5g4", "h8h5"]],
            [["g2g3", "a8c8", "e5d3", "e7f8"], ["e5d3", "a8c8", "g2g3", "e7f8"]],
        ];

        let board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

        for (i, [moves_a, moves_b]) in MOVES.iter().enumerate() {
            let mut board_a = board.clone();
            let mut board_b = board.clone();
            for mv in moves_a {
                board_a.simple_try_play(Move::from_str(mv, &board_a).unwrap());
            }
            for mv in moves_b {
                board_b.simple_try_play(Move::from_str(mv, &board_b).unwrap());
            }

            assert_eq!(ZobristHash::complete(&board_a), ZobristHash::complete(&board_b), "Test {}", i + 1);
        }
    }
}
