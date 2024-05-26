use crate::{movegen::board_rep::START_FEN, search::search_manager::SearchManager};

pub struct UciHandler {
    search_manager: SearchManager,
}

impl UciHandler {
    pub fn new() -> Self {
        Self {
            search_manager: SearchManager::new(START_FEN)
        }
    }
}