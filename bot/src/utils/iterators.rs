pub struct GapSequence64 {
    current: u64,
    end: u64,
    gap: u64,
}

impl GapSequence64 {
    pub fn new(start: u64, end: u64, gap: u64) -> Self {
        // start is allowed to be greater than end. in this case `next` will return None immediately
        Self {
            current: start,
            end,
            gap,
        }
    }
}

impl Iterator for GapSequence64 {
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

pub struct GapSequence32 {
    current: u32,
    end: u32,
    gap: u32,
}

impl GapSequence32 {
    pub fn new(start: u32, end: u32, gap: u32) -> Self {
        Self {
            current: start,
            end,
            gap,
        }
    }
}

impl Iterator for GapSequence32 {
    type Item = (u32, u32);

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
