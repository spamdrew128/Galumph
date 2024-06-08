use bytemuck::Zeroable;

use super::rng::Rng;

#[derive(Zeroable)]
#[repr(C)]
struct ZobristKeys {
    pieces: [[[u64; 64 as usize]; 6 as usize]; 2 as usize],
    castling: [u64; 16],
    ep_file: [u64; 8],
    black_to_move: u64,
}

fn get_zobrist_bytes() -> Box<[u8;]> {
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

    let bytes: Box<ZobristKeys> = bytemuck::allocation::try_cast_box(res).unwrap();
    bytes
}
