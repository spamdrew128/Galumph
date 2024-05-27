use crate::search::search_manager::SearchManager;

enum Token {
    
}

pub struct UciHandler {
    search_manager: SearchManager,
}

fn end_of_transmission(buffer: &str) -> bool {
    buffer
        .chars()
        .next()
        .map_or(false, |c| c == char::from(0x04))
}

fn kill_program() {
    std::process::exit(0);
}

impl UciHandler {
    pub fn new() -> Self {
        Self {
            search_manager: SearchManager::new()
        }
    }
}