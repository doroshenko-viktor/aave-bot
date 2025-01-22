pub struct GapSequence {
    current: u64,
    end: u64,
    gap: u64,
}

impl GapSequence {
    pub fn new(start: u64, end: u64, gap: u64) -> Self {
        Self {
            current: start,
            end,
            gap,
        }
    }
}

impl Iterator for GapSequence {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let lower = self.current;
        let higher = self.gap - 1;
        self.current += if higher > self.end { self.end } else { higher };

        Some((lower, self.current))
    }
}
