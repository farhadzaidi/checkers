use std::time::Duration;

/// How long a search may run: a think time, and optionally a fixed number of iterations
#[derive(Clone, Copy, Debug)]
pub struct SearchLimits {
    pub time: Duration,
    // Stop after this many iterations even with time left, so tests do the same work every run
    pub iterations: Option<u64>,
}

impl SearchLimits {
    /// Think for the given time, with no iteration limit
    pub fn time(time: Duration) -> Self {
        Self {
            time,
            iterations: None,
        }
    }
}
