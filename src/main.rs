// remove this later!
#![allow(dead_code)]

mod movegen;
mod search;
mod uci;
mod util_macros;

fn main() {
    movegen::perft::speed_test();
    let mut uci_handler = uci::uci_handler::UciHandler::new();

    loop {
        uci_handler.respond();
    }
}
