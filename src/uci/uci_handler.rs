use crate::{
    search::search_manager::SearchManager,
    uci::{
        constants::{AUTHOR, NAME, VERSION},
        setoption::display_options,
        uci_input::UciCommand,
    },
};

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
            Quit => kill_program(),
            Uci => {
                println!("id name {} v{}", NAME, VERSION);
                println!("id author {}", AUTHOR);
                display_options();
                println!("uciok");
            }
            IsReady => println!("readyok"),
            UciNewGame => self.search_manager = SearchManager::new(),
            Position(board) => self.search_manager.update_board(&board),
            Go(_) => self.search_manager.start_search(),
            _ => println!("Unrecognized Command"),
        };
    }
}
