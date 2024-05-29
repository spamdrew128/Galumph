use crate::movegen::board_rep::{Bitboard, Direction, Square};

const TABLE_SIZE: usize = 107481;
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
    hash_table: [Bitboard; TABLE_SIZE],
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
