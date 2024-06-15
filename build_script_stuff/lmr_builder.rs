use std::mem::transmute;

pub const MAX_MOVECOUNT: u8 = u8::MAX;
pub const MAX_PLY: u8 = i8::MAX as u8;
type Depth = u8;

type ReductionTable = [[Depth; MAX_MOVECOUNT as usize]; MAX_PLY as usize];
const BYTES: usize = std::mem::size_of::<ReductionTable>();

pub fn get_lmr_bytes() -> Box<[u8; BYTES]> {
    const LMR_BASE: f64 = 0.77;
    const LMR_DIVISOR: f64 = 3.0;

    let mut reduction_table: ReductionTable = [[0; MAX_MOVECOUNT as usize]; MAX_PLY as usize];
    for d in 0..MAX_PLY {
        for m in 0..(MAX_MOVECOUNT as u32) {
            let depth = f64::from(d.max(1));
            let move_count = f64::from(m.max(1));
            reduction_table[d as usize][m as usize] =
                (LMR_BASE + depth.ln() * move_count.ln() / LMR_DIVISOR) as Depth;
        }
    }

    let bytes: [u8; BYTES] = unsafe { transmute(reduction_table) };
    Box::from(bytes)
}
