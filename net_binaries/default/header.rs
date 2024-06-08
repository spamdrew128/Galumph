/*
    Do not modify this header.
    If you want to add your own net, do so in
    the user folder, and modify the header there.
*/

const INPUT_SIZE: usize = 64 * 6 * 2;
const L1_SIZE: usize = 64;

const L1_SCALE: i16 = 255;
const OUTPUT_SCALE: i16 = 64;

fn activation(sum: i16) -> i32 {
    i32::from(sum.clamp(0, L1_SCALE))
}