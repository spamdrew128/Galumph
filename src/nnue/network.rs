const NUM_BUCKETS: usize = 1;

#[repr(C)]
pub struct Network {
    feature_weights: [Accumulator; 768 * NUM_BUCKETS],
    feature_bias: Accumulator,
    output_weights: [Accumulator; 2],
    output_bias: i16,
}