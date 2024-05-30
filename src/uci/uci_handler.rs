use crate::{
    search::search_manager::{SearchConfig, SearchLimit, SearchManager},
    uci::{
        constants::{AUTHOR, NAME, VERSION},
        setoption::display_options,
        uci_input::{GoArg, UciCommand},
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
            Go(args) => {
                let mut config = SearchConfig::new();

                for arg in args {
                    match arg {
                        GoArg::Time(c, ms) => config.time[c.as_index()] = ms,
                        GoArg::Inc(c, ms) => config.inc[c.as_index()] = ms,
                        GoArg::MoveTime(ms) => config.limits.push(SearchLimit::MoveTime(ms)),
                        GoArg::Nodes(nodes) => config.limits.push(SearchLimit::Nodes(nodes)), 
                        GoArg::Depth(depth) => config.limits.push(SearchLimit::Depth(depth)), 
                        GoArg::MovesToGo(cnt) => config.moves_to_go = Some(cnt),
                        GoArg::Infinite => {
                            config = SearchConfig::new();
                            break;
                        }
                        _ => println!("Unrecognized Go Arg"),
                    }
                }

                self.search_manager.start_search(&config);
            }
            _ => println!("Unrecognized Command"),
        };
    }
}
