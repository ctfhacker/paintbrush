#[derive(Copy, Clone, Debug)]
pub struct Stats {
    pub start_time: usize
}

impl Stats {
    pub const fn new() -> Self {
        Self {
            start_time: 0
        }
    }
}
