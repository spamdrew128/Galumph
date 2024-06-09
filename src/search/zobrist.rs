#[repr(C)]
struct ZobristKeys {
    pieces: [[[u64; 64 as usize]; 6 as usize]; 2 as usize],
    castling: [u64; 16],
    ep_file: [u64; 8],
    black_to_move: u64,
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::search::zobrist::ZobristKeys;

    #[test]
    fn basic_test() {
        print!("P");
    }
}
