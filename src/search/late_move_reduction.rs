use crate::move_generation::movegen::MovePicker;

use super::constants::{Depth, MAX_PLY};

const LMR_TABLE: [[Depth; MovePicker::SIZE]; MAX_PLY as usize] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/lmr_init.bin"))) };

pub const fn get_lmr_reduction(depth: Depth, move_count: i32) -> Depth {
    LMR_TABLE[depth as usize][move_count as usize]
}
