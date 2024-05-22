use crate::{
    board_rep::{Bitboard, Direction, Square},
    magic_tables::{BISHOP_MAGICS, ROOK_MAGICS},
};

#[derive(Debug)]
struct MagicEntry {
    mask: Bitboard,
    magic: u64,
    shift: u8,
    table_offset: usize,
}

impl MagicEntry {
    const EMPTY: Self = Self {
        mask: Bitboard::EMPTY,
        magic: 0,
        shift: 0,
        table_offset: 0,
    };

    // const R_SHIFT: u8 = Square::CNT - 12;
    // const B_SHIFT: u8 = Square::CNT - 9;

    const fn new(mask: Bitboard, magic: u64, table_offset: usize) -> Self {
        let shift = Square::CNT - mask.popcount();
        Self {
            mask,
            magic,
            shift,
            table_offset,
        }
    }

    const fn hash_index(&self, blockers: Bitboard) -> usize {
        (((blockers.as_u64().wrapping_mul(self.magic)) >> self.shift) as usize) + self.table_offset
    }
}

pub struct MagicHashTable {
    rook_entries: [MagicEntry; Square::CNT as usize],
    bishop_entries: [MagicEntry; Square::CNT as usize],
    hash_table: [Bitboard; crate::magic_tables::TABLE_SIZE],
}

const fn generate_mask(sq: Square, directions: &[Direction; 4]) -> Bitboard {
    let mut result = Bitboard::EMPTY;
    let start = sq.as_bitboard();

    let mut i = 0;
    while i < 4 {
        let dir = directions[i];
        let mut bitset = start.shift(dir, 1);

        while bitset.shift(dir, 1).not_empty() {
            result = result.or(bitset);
            bitset = bitset.shift(dir, 1);
        }

        i += 1;
    }

    result
}

const fn generate_attacks(sq: Square, blockers: Bitboard, directions: &[Direction; 4]) -> Bitboard {
    let mut result = Bitboard::EMPTY;
    let availible = blockers.not();
    let start = sq.as_bitboard();

    let mut i = 0;
    while i < 4 {
        let mut bitset = start;
        let dir = directions[i];

        while bitset.overlaps(availible) {
            bitset = bitset.shift(dir, 1);
            result = result.or(bitset);
        }

        i += 1;
    }

    result
}

impl MagicHashTable {
    pub const fn generate() -> Self {
        let rook_dirs = &[Direction::N, Direction::E, Direction::S, Direction::W];
        let bishop_dirs = &[Direction::NE, Direction::SE, Direction::SW, Direction::NW];

        let mut rook_entries = [MagicEntry::EMPTY; Square::CNT as usize];
        let mut bishop_entries = [MagicEntry::EMPTY; Square::CNT as usize];
        let mut hash_table = [Bitboard::EMPTY; crate::magic_tables::TABLE_SIZE];

        let mut offset = 0;

        // rooks
        let mut i = 0;
        while i < Square::CNT {
            let sq = Square::new(i);
            let (magic, start, end) = ROOK_MAGICS[sq.as_index()];
            let size = end - start;

            rook_entries[sq.as_index()] =
                MagicEntry::new(generate_mask(sq, rook_dirs), magic, offset);
            offset += size;

            // fill_rook_entries
            let r_entry = &rook_entries[i as usize];
            let set = r_entry.mask.as_u64();
            let mut subset: u64 = 0;
            loop {
                let blockers = Bitboard::new(subset);
                let index = r_entry.hash_index(blockers);
                assert!(index < size);

                hash_table[index] = generate_attacks(sq, blockers, rook_dirs);

                subset = subset.wrapping_sub(set) & set;
                if subset == 0 {
                    break;
                }
            }

            i += 1;
        }

        // bishops
        i = 0;
        while i < Square::CNT {
            let sq = Square::new(i);
            let (magic, start, end) = BISHOP_MAGICS[sq.as_index()];
            let size = end - start;

            bishop_entries[sq.as_index()] =
                MagicEntry::new(generate_mask(sq, bishop_dirs), magic, offset);
            offset += size;

            // bishop hash entries
            let b_entry = &bishop_entries[i as usize];
            let set = b_entry.mask.as_u64();
            let mut subset: u64 = 0;
            loop {
                let blockers = Bitboard::new(subset);
                let index = b_entry.hash_index(blockers);
                assert!(index < size);

                hash_table[index] = generate_attacks(sq, blockers, rook_dirs);

                subset = subset.wrapping_sub(set) & set;
                if subset == 0 {
                    break;
                }
            }

            i += 1;
        }

        assert!(offset == crate::magic_tables::TABLE_SIZE);

        Self {
            rook_entries,
            bishop_entries,
            hash_table,
        }
    }
}
