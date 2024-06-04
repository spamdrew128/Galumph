use bytemuck::{AnyBitPattern, NoUninit, Pod, Zeroable};

use super::rng::Rng;

const INPUT_SIZE: usize = 64 * 6 * 2;
const L1_SIZE: usize = 64;

const INPUT_SCALE: i16 = 255;
const OUTPUT_SCALE: i16 = 64;

#[derive(Debug, Zeroable, Pod, Copy, Clone)]
#[repr(C, align(64))]
pub struct L1Params([i16; L1_SIZE]);

#[derive(Debug, Zeroable, NoUninit, Copy, Clone)]
#[repr(C)]
pub struct Network {
    l1_weights: [L1Params; INPUT_SIZE],
    l1_biases: L1Params,
    output_weights: [L1Params; 2],
    output_bias: i16,
    _padding: [u8; 62],
}

#[derive(Debug, Copy, Clone, AnyBitPattern)]
#[repr(C, align(64))]
pub struct NetBytes {
    pub bytes: [u8; std::mem::size_of::<Network>()],
}

pub fn get_random_nnue_bytes() -> Box<NetBytes> {
    let mut rng = Rng::new();

    const ZERO_L1: L1Params = L1Params([0; L1_SIZE]);

    fn rand_l1(rng: &mut Rng, scale: i16) -> L1Params {
        let mut res = ZERO_L1;

        for v in res.0.iter_mut() {
            *v = rng.rand_i16() % scale;
        }
        res
    }

    let mut res: Box<Network> = bytemuck::allocation::zeroed_box();

    for v in res.l1_weights.iter_mut() {
        *v = rand_l1(&mut rng, INPUT_SCALE);
    }

    res.l1_biases = rand_l1(&mut rng, INPUT_SCALE);

    for v in res.output_weights.iter_mut() {
        *v = rand_l1(&mut rng, OUTPUT_SCALE);
    }

    res.output_bias = rng.rand_i16() % OUTPUT_SCALE;

    let net_bytes: Box<NetBytes> = bytemuck::allocation::try_cast_box(res).unwrap();
    net_bytes
}
