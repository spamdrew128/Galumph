/*
    This file should list the parameters of the network.
    If you input your own network, be sure to change this file.
*/

const INPUT_SIZE: usize = 64 * 6 * 2;
const L1_SIZE: usize = 768;

const L1_SCALE: i16 = 255;
const OUTPUT_SCALE: i16 = 64;

fn activation(sum: i16) -> i32 {
    i32::from(sum.clamp(0, L1_SCALE))
}