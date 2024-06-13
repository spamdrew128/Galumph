use crate::{move_generation::chess_move::Move, search::constants::{Ply, MAX_PLY}};

#[derive(Debug, Clone)]
pub struct Killers {
    moves: [Move; MAX_PLY as usize],
}

impl Killers {
    pub const fn new() -> Self {
        Self {
            moves: [Move::NULL; MAX_PLY as usize],
        }
    }

    pub fn update(&mut self, mv: Move, ply: Ply) {
        self.moves[ply as usize] = mv;
    }

    pub const fn killer(&self, ply: Ply) -> Move {
        self.moves[ply as usize]
    }
}
