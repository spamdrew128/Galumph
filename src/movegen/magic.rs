use crate::movegen::{
    board_rep::{Bitboard, Direction, Square},
    magic_tables::{self, BISHOP_MAGICS, ROOK_MAGICS},
};

const ROOK_DIRS: [Direction; 4] = [Direction::N, Direction::E, Direction::S, Direction::W];
const BISHOP_DIRS: [Direction; 4] = [Direction::NE, Direction::SE, Direction::SW, Direction::NW];

#[derive(Debug)]
#[repr(C)]
struct MagicEntry {
    shift: u8,
    table_offset: usize,
    magic: u64,
    mask: Bitboard,
}

impl MagicEntry {
    const EMPTY: Self = Self {
        mask: Bitboard::EMPTY,
        magic: 0,
        shift: 0,
        table_offset: 0,
    };

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

#[repr(C)]
pub struct MagicHashTable {
    rook_entries: [MagicEntry; Square::CNT as usize],
    bishop_entries: [MagicEntry; Square::CNT as usize],
    hash_table: [Bitboard; magic_tables::TABLE_SIZE],
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
    pub const fn construct() -> Self {
        let mut rook_entries = [MagicEntry::EMPTY; Square::CNT as usize];
        let mut bishop_entries = [MagicEntry::EMPTY; Square::CNT as usize];
        let mut hash_table = [Bitboard::EMPTY; magic_tables::TABLE_SIZE];

        let mut offset = 0;

        // rooks
        let mut i = 0;
        while i < Square::CNT {
            let sq = Square::new(i);
            let (magic, start, end) = ROOK_MAGICS[sq.as_index()];
            let size = end - start;

            rook_entries[sq.as_index()] =
                MagicEntry::new(generate_mask(sq, &ROOK_DIRS), magic, offset);
            offset += size;

            // fill_rook_entries
            let r_entry = &rook_entries[i as usize];
            let set = r_entry.mask.as_u64();
            let mut subset: u64 = 0;
            loop {
                let blockers = Bitboard::new(subset);
                let index = r_entry.hash_index(blockers);
                assert!(index < offset);

                hash_table[index] = generate_attacks(sq, blockers, &ROOK_DIRS);

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
                MagicEntry::new(generate_mask(sq, &BISHOP_DIRS), magic, offset);
            offset += size;

            // bishop hash entries
            let b_entry = &bishop_entries[i as usize];
            let set = b_entry.mask.as_u64();
            let mut subset: u64 = 0;
            loop {
                let blockers = Bitboard::new(subset);
                let index = b_entry.hash_index(blockers);
                assert!(index < offset);

                hash_table[index] = generate_attacks(sq, blockers, &BISHOP_DIRS);

                subset = subset.wrapping_sub(set) & set;
                if subset == 0 {
                    break;
                }
            }

            i += 1;
        }

        assert!(offset == magic_tables::TABLE_SIZE);

        Self {
            rook_entries,
            bishop_entries,
            hash_table,
        }
    }

    pub fn rook_attack_set(&self, sq: Square, occupied: Bitboard) -> Bitboard {
        let entry = &self.rook_entries[sq.as_index()];
        let blockers = entry.mask & occupied;
        self.hash_table[entry.hash_index(blockers)]
    }

    pub fn bishop_attack_set(&self, sq: Square, occupied: Bitboard) -> Bitboard {
        let entry = &self.bishop_entries[sq.as_index()];
        let blockers = entry.mask & occupied;
        self.hash_table[entry.hash_index(blockers)]
    }
}

#[cfg(test)]
mod tests {
    use crate::movegen::{
        attacks,
        board_rep::{Bitboard, Square},
    };

    use super::{generate_attacks, generate_mask, BISHOP_DIRS, ROOK_DIRS};

    #[test]
    fn exhaustive_slider_test() {
        for i in 0..Square::CNT {
            let sq = Square::new(i);
            let r_mask = generate_mask(sq, &ROOK_DIRS).as_u64();
            let b_mask = generate_mask(sq, &BISHOP_DIRS).as_u64();

            // ROOK TEST
            let set = r_mask;
            let mut subset = 0;
            loop {
                let blockers = Bitboard::new(subset);
                let attacks = generate_attacks(sq, blockers, &ROOK_DIRS);

                assert_eq!(attacks::rook(sq, blockers), attacks);

                subset = subset.wrapping_sub(set) & set;
                if subset == 0 {
                    break;
                }
            }

            // BISHOP TEST
            let set = b_mask;
            let mut subset = 0;
            loop {
                let blockers = Bitboard::new(subset);
                let attacks = generate_attacks(sq, blockers, &BISHOP_DIRS);

                assert_eq!(attacks::bishop(sq, blockers), attacks);

                subset = subset.wrapping_sub(set) & set;
                if subset == 0 {
                    break;
                }
            }
        }
    }
}
