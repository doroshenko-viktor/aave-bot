pub struct GapSequence32 {
    current: u32,
    end: u32,
    gap: u32,
}

impl GapSequence32 {
    pub fn new(start: u32, end: u32, gap: u32) -> Self {
        Self { current: start, end, gap }
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

pub struct GapSequence {
    current: u64,
    end: u64,
    previous: u64,
    interval: u64,
    max_interval: u64,
    interval_error: bool,
}

impl GapSequence {
    pub fn new(start: u64, end: u64, max_interval: u64) -> Self {
        // start is allowed to be greater than end. in this case `next` will return None immediately
        Self {
            current: start,
            previous: start,
            end,
            interval: max_interval,
            max_interval,
            interval_error: false,
        }
    }

    pub fn error_interval(&mut self) {
        self.interval_error = true;
        self.interval = self.interval / 2;
    }

    pub fn success_interval(&mut self) {
        self.interval_error = false;

        self.current = self.current + self.interval + 1;
        self.previous = self.current;

        self.interval = self.interval * 2;
        if self.interval > self.max_interval {
            self.interval = self.max_interval;
        }
    }
}

impl Iterator for GapSequence {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let (lower, mut higher) = if !self.interval_error {
            (
                self.current,
                self.current + self.interval,
            )
        } else {
            (
                self.previous,
                self.previous + self.interval,
            )
        };

        if higher > self.end {
            higher = self.end;
        }

        Some((lower, higher))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_first_step() {
        let mut gap_sequence = GapSequence::new(0, 40, 10);
        assert_eq!(gap_sequence.next(), Some((0, 10)));
    }

    #[test]
    fn test_full_run() {
        let mut gap_sequence = GapSequence::new(0, 40, 10);
        assert_eq!(gap_sequence.next(), Some((0, 10)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((11, 21)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((22, 32)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((33, 40)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), None);
    }

    #[test]
    fn test_one_error_loop() {
        let mut gap_sequence = GapSequence::new(0, 25, 10);
        assert_eq!(gap_sequence.next(), Some((0, 10)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((11, 21)));
        gap_sequence.error_interval();
        assert_eq!(gap_sequence.next(), Some((11, 16)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((17, 25)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), None);
    }

    #[test]
    fn test_multiple_error_loop() {
        let mut gap_sequence = GapSequence::new(0, 25, 10);
        assert_eq!(gap_sequence.next(), Some((0, 10)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((11, 21)));
        gap_sequence.error_interval();
        assert_eq!(gap_sequence.next(), Some((11, 16)));
        gap_sequence.error_interval();
        assert_eq!(gap_sequence.next(), Some((11, 13)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((14, 18)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), Some((19, 25)));
        gap_sequence.success_interval();
        assert_eq!(gap_sequence.next(), None);
    }
}
