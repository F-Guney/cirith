use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    max_requests: u64,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window_secs: u64) -> RateLimiter {
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
}
