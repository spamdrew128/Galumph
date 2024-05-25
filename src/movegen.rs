use crate::chess_move::Move;

#[derive(Debug, Copy, Clone)]
pub struct ScoredMove {
    mv: Move,
    score: i16,
}

impl ScoredMove {
    const fn new() -> Self {
        Self {
            mv: Move::NULL,
            score: 0,
        }
    }
}

pub struct MovePicker {
    list: [Move; Self::SIZE],
    idx: usize,
}

impl MovePicker {
    const SIZE: usize = u8::MAX as usize;

    fn new() -> Self {
        todo!()
    }
}
