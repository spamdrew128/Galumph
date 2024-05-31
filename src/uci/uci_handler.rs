use std::thread;

use crate::{
    search::{
        constants::Milliseconds,
        search_manager::{self, SearchConfig, SearchLimit, SearchManager},
    },
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
    stored_command: Option<UciCommand>,

    // options
    overhead: Milliseconds,
}

impl UciHandler {
    pub fn new() -> Self {
        Self {
            search_manager: SearchManager::new(),
            stored_command: None,
            overhead: 0,
        }
    }

    fn respond_while_searching() -> Option<UciCommand> {
        loop {
            let command = UciCommand::recieve_valid();

            if search_manager::stop_flag_is_set() {
                return Some(command);
            }

            use UciCommand::*;
            match command {
                IsReady => println!("readyok"),
                Quit => kill_program(),
                Stop => {
                    search_manager::set_stop_flag();
                    return None;
                }
                _ => {
                    eprintln!("Cannot handle this command while searching");
                }
            }
        }
    }

    pub fn respond(&mut self) {
        let stored = self.stored_command.clone();
        self.stored_command = None;

        let command = if let Some(cmd) = stored {
            cmd
        } else {
            UciCommand::recieve_valid()
        };

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
                let mut config = SearchConfig::new(self.overhead);

                for arg in args {
                    match arg {
                        GoArg::Time(c, ms) => config.time[c.as_index()] = ms,
                        GoArg::Inc(c, ms) => config.inc[c.as_index()] = ms,
                        GoArg::MovesToGo(cnt) => config.moves_to_go = Some(cnt),
                        GoArg::MoveTime(ms) => config.limits.push(SearchLimit::MoveTime(ms)),
                        GoArg::Nodes(nodes) => config.limits.push(SearchLimit::Nodes(nodes)),
                        GoArg::Depth(depth) => config.limits.push(SearchLimit::Depth(depth)),
                        GoArg::Infinite => config.limits.push(SearchLimit::Infinite),
                        _ => eprintln!("Unrecognized Go Arg"),
                    }
                }

                if config.limits.is_empty() {
                    config.limits.push(SearchLimit::Standard);
                }

                search_manager::clear_stop_flag();
                thread::scope(|s| {
                    s.spawn(|| {
                        self.search_manager.start_search(&config);
                    });

                    self.stored_command = Self::respond_while_searching();
                });
            }
            Stop => eprintln!("Uneeded Stop: Not Searching"),
            SetOptionOverHead(time) => self.overhead = Milliseconds::from(time),
            _ => eprintln!("Unrecognized Command"),
        };
    }
}
