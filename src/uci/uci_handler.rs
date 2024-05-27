use crate::{search::search_manager::SearchManager, uci::uci_input::UciCommand};

pub fn kill_program() {
    std::process::exit(0);
}

pub struct UciHandler {
    search_manager: SearchManager,
}

impl UciHandler {
    pub fn new() -> Self {
        Self {
            search_manager: SearchManager::new(),
        }
    }

    pub fn respond(&mut self) {
        let command = UciCommand::recieve_valid();

        use UciCommand::*;
        match command {
            _ => (),
        };
    }
}
