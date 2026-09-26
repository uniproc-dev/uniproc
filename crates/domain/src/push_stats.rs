use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_secs(30);

pub struct PushStats<T> {
    name: &'static str,
    last: Option<T>,
    total: u64,
    same: u64,
    since: Instant,
}

impl<T: PartialEq> PushStats<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            last: None,
            total: 0,
            same: 0,
            since: Instant::now(),
        }
    }

    pub fn note(&mut self, value: T) {
        self.total += 1;
        if self.last.as_ref() == Some(&value) {
            self.same += 1;
        }
        self.last = Some(value);
        if self.since.elapsed() >= WINDOW {
            tracing::warn!(
                reducer = self.name,
                pushes = self.total,
                unchanged = self.same,
                "push stats over 30s"
            );
            self.total = 0;
            self.same = 0;
            self.since = Instant::now();
        }
    }
}

impl<T> std::fmt::Debug for PushStats<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PushStats").field("name", &self.name).finish()
    }
}
