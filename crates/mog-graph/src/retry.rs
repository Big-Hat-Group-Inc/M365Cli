use rand::Rng;
use std::time::Duration;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(16),
            jitter_factor: 0.2,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given attempt (0-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.base_delay.as_millis() as f64 * 2.0_f64.powi(attempt as i32);
        let base_ms = base_ms.min(self.max_delay.as_millis() as f64);

        // Apply jitter ±jitter_factor
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-self.jitter_factor..=self.jitter_factor);
        let jittered = base_ms * (1.0 + jitter);

        Duration::from_millis(jittered.max(0.0) as u64)
    }

    /// Parse Retry-After header value (seconds or HTTP-date)
    pub fn parse_retry_after(value: &str) -> Option<Duration> {
        // Try parsing as seconds first
        if let Ok(secs) = value.trim().parse::<u64>() {
            return Some(Duration::from_secs(secs));
        }
        // Try parsing as HTTP-date
        if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(value.trim()) {
            let now = chrono::Utc::now();
            let target = dt.with_timezone(&chrono::Utc);
            if target > now {
                let diff = (target - now).num_seconds().max(0) as u64;
                return Some(Duration::from_secs(diff));
            }
            return Some(Duration::from_secs(0));
        }
        None
    }
}

/// Whether a status code is retryable
pub fn is_retryable_status(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_increases() {
        let config = RetryConfig {
            jitter_factor: 0.0,
            ..Default::default()
        };
        let d0 = config.delay_for_attempt(0);
        let d1 = config.delay_for_attempt(1);
        let d2 = config.delay_for_attempt(2);
        assert!(d1 > d0);
        assert!(d2 > d1);
    }

    #[test]
    fn test_parse_retry_after_seconds() {
        assert_eq!(
            RetryConfig::parse_retry_after("5"),
            Some(Duration::from_secs(5))
        );
    }

    #[test]
    fn test_retryable_status() {
        assert!(is_retryable_status(429));
        assert!(is_retryable_status(503));
        assert!(is_retryable_status(500));
        assert!(is_retryable_status(502));
        assert!(is_retryable_status(504));
        assert!(!is_retryable_status(200));
        assert!(!is_retryable_status(404));
        assert!(!is_retryable_status(401));
    }

    #[test]
    fn test_delay_capped_at_max() {
        let config = RetryConfig {
            jitter_factor: 0.0,
            max_delay: Duration::from_secs(16),
            ..Default::default()
        };
        // Attempt 10 should be capped at max_delay
        let d = config.delay_for_attempt(10);
        assert!(d <= Duration::from_secs(16));
    }

    #[test]
    fn test_exponential_backoff_values() {
        let config = RetryConfig {
            jitter_factor: 0.0,
            base_delay: Duration::from_secs(1),
            ..Default::default()
        };
        // attempt 0 => 1s, attempt 1 => 2s, attempt 2 => 4s
        assert_eq!(config.delay_for_attempt(0), Duration::from_secs(1));
        assert_eq!(config.delay_for_attempt(1), Duration::from_secs(2));
        assert_eq!(config.delay_for_attempt(2), Duration::from_secs(4));
    }

    #[test]
    fn test_parse_retry_after_invalid() {
        assert!(RetryConfig::parse_retry_after("not-a-number").is_none());
    }
}
