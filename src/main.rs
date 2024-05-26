// remove this later!
#![allow(dead_code)]

mod attacks;
mod board_rep;
mod chess_move;
mod magic;
mod magic_tables;
mod movegen;
mod perft;
mod util_macros;

fn main() {
    perft::speed_test();
}
