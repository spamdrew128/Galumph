use std::mem::{size_of, transmute};

use super::rng::Rng;

#[derive(Debug)]
#[repr(C)]
struct ZobristKeys {
    pieces: [[[u64; 64 as usize]; 6 as usize]; 2 as usize],
    castling: [u64; 16],
    ep_file: [u64; 8],
    black_to_move: u64,
}

impl ZobristKeys {
    fn new() -> Self {
        Self {
            pieces: [[[0; 64 as usize]; 6 as usize]; 2 as usize],
            castling: [0; 16],
            ep_file: [0; 8],
            black_to_move: 0,
        }
    }
}

pub fn get_zobrist_bytes() -> Box<[u8; size_of::<ZobristKeys>()]> {
    let mut res = ZobristKeys::new();
    let mut rng = Rng::new();

    res.pieces.iter_mut().flatten().flatten().for_each(|v| {
        *v = rng.rand_u64();
    });
    res.castling.iter_mut().for_each(|v| {
        *v = rng.rand_u64();
    });
    res.ep_file.iter_mut().for_each(|v| {
        *v = rng.rand_u64();
    });
    res.black_to_move = rng.rand_u64();

    let bytes: [u8; size_of::<ZobristKeys>()] = unsafe { transmute(res) };
    Box::from(bytes)
}
