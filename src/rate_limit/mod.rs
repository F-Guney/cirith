use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    max_requests: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        let mut requests = match self.requests.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        let instants = requests.entry(ip).or_insert_with(Vec::new);
        instants.retain(|t| now.duration_since(*t) <= window);

        if instants.len() < self.max_requests as usize {
            instants.push(now);
            true
        } else {
            false
        }
    }

    pub fn cleanup(&self) {
        let mut requests = match self.requests.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);

        for instant in requests.values_mut() {
            instant.retain(|t| now.duration_since(*t) <= window);
        }

        requests.retain(|_ip, instants| !instants.is_empty());
    }

    pub fn spawn_cleanup_task(self: Arc<Self>, interval_secs: u64) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                self.cleanup();
                tracing::debug!("Rate limiter cleanup completed");
            }
        });
    }
}
