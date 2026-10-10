use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<Bucket>>,
    capacity: u32,
    refill_period: Duration,
}

struct Bucket {
    tokens: f64,
    updated: Instant,
}

impl RateLimiter {
    pub fn per_minute(limit: u32) -> Self {
        Self::new(limit, Duration::from_secs(60))
    }

    pub fn new(capacity: u32, refill_period: Duration) -> Self {
        let capacity = capacity.max(1);
        let refill_period = refill_period.max(Duration::from_nanos(1));
        Self {
            inner: Arc::new(Mutex::new(Bucket {
                tokens: f64::from(capacity),
                updated: Instant::now(),
            })),
            capacity,
            refill_period,
        }
    }

    pub async fn acquire(&self) {
        loop {
            let wait = {
                let mut bucket = self.inner.lock().await;
                let elapsed = bucket.updated.elapsed().as_secs_f64();
                let rate = f64::from(self.capacity) / self.refill_period.as_secs_f64();
                bucket.tokens = (bucket.tokens + elapsed * rate).min(f64::from(self.capacity));
                bucket.updated = Instant::now();
                if bucket.tokens >= 1.0 {
                    bucket.tokens -= 1.0;
                    return;
                }
                Duration::from_secs_f64((1.0 - bucket.tokens) / rate)
            };
            tokio::time::sleep(wait).await;
        }
    }
}
