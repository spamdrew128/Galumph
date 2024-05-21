use crate::board_rep::{Bitboard, Square};

#[derive(Debug, Clone, Copy)]
struct MagicEntry {
    mask: Bitboard,
    magic: u64,
    table_offset: usize,
}

impl MagicEntry {
    const fn generate_mask()

    const fn new_rook(sq: Square) -> Self {
        todo!()
    }
}

struct MagicHashTable {
    rook_entries: [MagicEntry; Square::CNT as usize],
    bishop_entries: [MagicEntry; Square::CNT as usize],
    hash_table: [Bitboard; crate::magic_tables::TABLE_SIZE as usize],
}
