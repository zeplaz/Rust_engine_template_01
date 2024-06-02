trait TimeTick {
    fn new_tick(&mut self);
    fn calculate_day_ticks(&self) -> usize; // Include &self to access instance data
}

struct GameTime {
    current_tick: usize,
    ticks_per_day: usize,
}

impl TimeTick for GameTime {
    fn new_tick(&mut self) {
        self.current_tick += 1;
    }

    fn calculate_day_ticks(&self) -> usize {
        self.current_tick / self.ticks_per_day
    }
}