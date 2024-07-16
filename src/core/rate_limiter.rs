use std::time::Duration;
use tokio::time::Instant;

pub struct RateLimiter {
    pub tokens: f64,
    pub last_refill: Instant,
    pub rate: f64,
    pub capacity: f64,
}

impl RateLimiter {
    pub fn new(rate: f64, capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            rate,
            capacity,
        }
    }

    pub async fn acquire(&mut self) {
        let now = Instant::now();
        let elapse = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapse * self.rate).min(self.capacity);
        self.last_refill = now;

        if self.tokens < 1.0 {
            let wait_time = (1.0 - self.tokens) / self.rate;
            tokio::time::sleep(Duration::from_secs_f64(wait_time)).await;
            self.tokens = 0.0
        } else {
            self.tokens -= 1.0
        }
    }
}
