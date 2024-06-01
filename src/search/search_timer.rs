use std::time::Instant;

use super::constants::Milliseconds;

#[derive(Debug, Copy, Clone)]
pub struct SearchTimer {
    timer: Instant,
    hard_limit: u128,
}

impl SearchTimer {
    pub fn new(hard_limit: Milliseconds) -> Self {
        Self {
            timer: Instant::now(),
            hard_limit: hard_limit.saturating_mul(1000),
        }
    }

    pub fn is_hard_expired(&self) -> bool {
        (self.timer.elapsed().as_micros()) > self.hard_limit
    }
}
