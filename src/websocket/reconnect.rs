//! Reconnection strategy with exponential backoff

use std::time::Duration;

/// Strategy for reconnecting WebSocket connections with exponential backoff
///
/// When a WebSocket connection fails, this strategy determines:
/// - Whether to retry (based on max_retries)
/// - How long to wait before retrying (exponential backoff)
///
/// # Business Logic
///
/// Exponential backoff prevents hammering the server during outages:
/// - Attempt 1: Wait 1 second
/// - Attempt 2: Wait 2 seconds
/// - Attempt 3: Wait 4 seconds
/// - Attempt 4: Wait 8 seconds
/// - ... up to max_delay cap
///
/// # Example
///
/// ```rust,no_run
/// use arb_bot::websocket::ReconnectionStrategy;
/// use std::time::Duration;
///
/// // Retry up to 10 times with exponential backoff
/// let strategy = ReconnectionStrategy::exponential_backoff();
///
/// // Or custom strategy
/// let custom = ReconnectionStrategy::new(
///     Some(5),                    // Max 5 retries
///     Duration::from_secs(1),     // Start with 1 second
///     Duration::from_secs(60),    // Cap at 60 seconds
/// );
/// ```
#[derive(Debug, Clone)]
pub struct ReconnectionStrategy {
    /// Maximum number of retries (None = retry forever)
    pub max_retries: Option<u32>,
    /// Current retry attempt number
    pub current_retry: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay cap (prevents waiting too long)
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (typically 2.0)
    pub multiplier: f64,
}

impl ReconnectionStrategy {
    /// Create a new reconnection strategy
    ///
    /// # Parameters
    ///
    /// - `max_retries`: Maximum number of retry attempts (None = infinite retries)
    /// - `initial_delay`: Initial wait time before first retry
    /// - `max_delay`: Maximum wait time cap (exponential won't exceed this)
    pub fn new(
        max_retries: Option<u32>,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            max_retries,
            current_retry: 0,
            initial_delay,
            max_delay,
            multiplier: 2.0,
        }
    }

    /// Create a default exponential backoff strategy
    ///
    /// - Max retries: 10
    /// - Initial delay: 1 second
    /// - Max delay: 60 seconds
    pub fn exponential_backoff() -> Self {
        Self::new(
            Some(10),
            Duration::from_secs(1),
            Duration::from_secs(60),
        )
    }

    /// Check if we should attempt another retry
    pub fn should_retry(&self) -> bool {
        match self.max_retries {
            Some(max) => self.current_retry < max,
            None => true, // Infinite retries
        }
    }

    /// Calculate the delay before the next retry attempt
    ///
    /// Uses exponential backoff: `initial_delay * (multiplier ^ current_retry)`
    /// Capped at `max_delay`.
    ///
    /// # Side Effect
    ///
    /// Increments `current_retry` counter.
    pub fn next_delay(&mut self) -> Duration {
        // Calculate exponential: multiplier ^ current_retry
        // Cap the exponent to prevent overflow (max ~30 for 2.0 multiplier)
        let exponent = self.current_retry.min(30) as i32;
        let multiplier_power = self.multiplier.powi(exponent);

        // Calculate delay, but ensure it doesn't overflow
        let delay_secs = self.initial_delay.as_secs_f64() * multiplier_power;

        self.current_retry += 1;

        // Cap at max_delay to prevent overflow
        let max_delay_secs = self.max_delay.as_secs_f64();
        let capped_secs = delay_secs.min(max_delay_secs);

        // Convert back to Duration (safe because we capped the value)
        Duration::from_secs_f64(capped_secs.min(u64::MAX as f64))
    }

    /// Reset the retry counter (called after successful connection)
    pub fn reset(&mut self) {
        self.current_retry = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_calculation() {
        let mut strategy = ReconnectionStrategy::new(
            Some(5),
            Duration::from_secs(1),
            Duration::from_secs(60),
        );

        // First retry: 1 * 2^0 = 1 second
        assert_eq!(strategy.next_delay().as_secs(), 1);
        assert_eq!(strategy.current_retry, 1);

        // Second retry: 1 * 2^1 = 2 seconds
        assert_eq!(strategy.next_delay().as_secs(), 2);
        assert_eq!(strategy.current_retry, 2);

        // Third retry: 1 * 2^2 = 4 seconds
        assert_eq!(strategy.next_delay().as_secs(), 4);
        assert_eq!(strategy.current_retry, 3);
    }

    #[test]
    fn test_max_delay_cap() {
        let mut strategy = ReconnectionStrategy::new(
            Some(10),
            Duration::from_secs(1),
            Duration::from_secs(10), // Max cap at 10 seconds
        );

        // After several retries, delay should cap at 10 seconds
        for _ in 0..10 {
            let delay = strategy.next_delay();
            assert!(delay <= Duration::from_secs(10));
        }
    }

    #[test]
    fn test_should_retry_with_max() {
        let mut strategy = ReconnectionStrategy::new(
            Some(3),
            Duration::from_secs(1),
            Duration::from_secs(60),
        );

        assert!(strategy.should_retry()); // Before first retry
        strategy.next_delay();
        assert!(strategy.should_retry()); // After first retry
        strategy.next_delay();
        assert!(strategy.should_retry()); // After second retry
        strategy.next_delay();
        assert!(!strategy.should_retry()); // After third retry (exceeded)
    }

    #[test]
    fn test_should_retry_infinite() {
        let mut strategy = ReconnectionStrategy::new(
            None, // No max retries
            Duration::from_secs(1),
            Duration::from_secs(60),
        );

        // Should always retry
        for _ in 0..100 {
            assert!(strategy.should_retry());
            strategy.next_delay();
        }
    }

    #[test]
    fn test_reset() {
        let mut strategy = ReconnectionStrategy::new(
            Some(5),
            Duration::from_secs(1),
            Duration::from_secs(60),
        );

        strategy.next_delay(); // Move to retry 1
        strategy.next_delay(); // Move to retry 2
        assert_eq!(strategy.current_retry, 2);

        strategy.reset();
        assert_eq!(strategy.current_retry, 0);

        // After reset, delay should be initial again
        let delay = strategy.next_delay();
        assert_eq!(delay.as_secs(), 1);
    }
}

