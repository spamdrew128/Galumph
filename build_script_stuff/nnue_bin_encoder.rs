use std::mem::transmute;

use bytemuck::{NoUninit, Pod, Zeroable};

use super::rng::Rng;

const INPUT_SIZE: usize = 64 * 6 * 2;
const L1_SIZE: usize = 64;

#[derive(Debug, Zeroable, Pod, Copy, Clone)]
#[repr(C, align(64))]
pub struct L1Params {
    vals: [i16; L1_SIZE],
}

#[derive(Debug, Zeroable, NoUninit, Copy, Clone)]
#[repr(C)]
pub struct Network {
    l1_weights: [L1Params; INPUT_SIZE],
    l1_biases: L1Params,
    output_weights: [L1Params; 2],
    output_biases: i16,
}

pub fn get_random_nnue_bytes() -> Box<[u8; std::mem::size_of::<Network>()]> {
    let mut rng = Rng::new();

    const ZERO_L1: L1Params = L1Params { vals: [0; L1_SIZE] };

    fn rand_l1(rng: &mut Rng) -> L1Params {
        let mut res = ZERO_L1;

        for v in res.vals.iter_mut() {
            *v = rng.rand_i16();
        }
        res
    }

    let mut res: Box<Network> = bytemuck::allocation::zeroed_box();

    for v in res.l1_weights.iter_mut() {
        *v = rand_l1(&mut rng);
    }

    res.l1_biases = rand_l1(&mut rng);

    for v in res.output_weights.iter_mut() {
        *v = rand_l1(&mut rng);
    }

    res.output_biases = rng.rand_i16();

    bytemuck::allocation::try_cast_box(res).unwrap()
}
