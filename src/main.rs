// remove this later!
#![allow(dead_code)]

mod move_generation;
mod nnue;
mod search;
mod uci;
mod util_macros;

fn main() {
    let mut uci_handler = uci::uci_handler::UciHandler::new();

    loop {
        uci_handler.respond();
    }
}
