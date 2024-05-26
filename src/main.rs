// remove this later!
#![allow(dead_code)]

mod movegen;
mod search;
mod uci;
mod util_macros;

fn main() {
    movegen::perft::speed_test();
    let _uci_handler = uci::uci_handler::UciHandler::new();
}
