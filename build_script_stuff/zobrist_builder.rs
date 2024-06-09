use std::mem::size_of;

use bytemuck::{NoUninit, Zeroable};

use super::rng::Rng;

#[derive(Debug, Zeroable, NoUninit, Copy, Clone)]
#[repr(C)]
struct ZobristKeys {
    pieces: [[[u64; 64 as usize]; 6 as usize]; 2 as usize],
    castling: [u64; 16],
    ep_file: [u64; 8],
    black_to_move: u64,
}

pub fn get_zobrist_bytes() -> Box<[u8; size_of::<ZobristKeys>()]> {
    let mut res: Box<ZobristKeys> = bytemuck::allocation::zeroed_box();
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

    let bytes: Box<[u8; size_of::<ZobristKeys>()]> = bytemuck::allocation::try_cast_box(res).unwrap();
    bytes
}
