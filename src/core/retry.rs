//! retry — Small shared retry helpers for network calls.

use std::future::Future;
use std::time::Duration;

pub const DEFAULT_RETRY_ATTEMPTS: usize = 3;
const DEFAULT_BACKOFF_MS: u64 = 200;

pub fn retry_blocking_with_backoff<T, E, F, B>(
    attempts: usize,
    backoff: B,
    operation: &mut F,
) -> Result<T, E>
where
    F: FnMut(usize) -> Result<T, E>,
    B: Fn(usize) -> Duration,
{
    let attempts = attempts.max(1);
    let mut last = operation(1);

    for attempt in 2..=attempts {
        match last {
            Ok(value) => return Ok(value),
            Err(_) => {
                std::thread::sleep(backoff(attempt - 1));
                last = operation(attempt);
            }
        }
    }

    last
}

pub async fn retry_async<T, E, F, Fut>(mut operation: F) -> Result<T, E>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut last = operation(1).await;

    for attempt in 2..=DEFAULT_RETRY_ATTEMPTS {
        match last {
            Ok(value) => return Ok(value),
            Err(_) => {
                tokio::time::sleep(default_backoff(attempt - 1)).await;
                last = operation(attempt).await;
            }
        }
    }

    last
}

fn default_backoff(attempt: usize) -> Duration {
    let shift = attempt.saturating_sub(1).min(8) as u32;
    Duration::from_millis(DEFAULT_BACKOFF_MS.saturating_mul(2_u64.saturating_pow(shift)))
}

/// 断路器状态
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// 断路器
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub failure_threshold: u8,
    pub recovery_timeout_ms: u64,
    pub half_open_max_requests: u8,
    failure_count: u8,
    state: CircuitState,
    last_failure_time: Option<std::time::Instant>,
    half_open_requests: u8,
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u8,
        recovery_timeout_ms: u64,
        half_open_max_requests: u8,
    ) -> Self {
        Self {
            failure_threshold,
            recovery_timeout_ms,
            half_open_max_requests,
            failure_count: 0,
            state: CircuitState::Closed,
            last_failure_time: None,
            half_open_requests: 0,
        }
    }

    pub fn call<T: Default, E, F>(&mut self, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match self.state {
            CircuitState::Open => {
                if self
                    .last_failure_time
                    .is_some_and(|t| t.elapsed().as_millis() as u64 >= self.recovery_timeout_ms)
                {
                    self.state = CircuitState::HalfOpen;
                    self.half_open_requests = 0;
                } else {
                    return Err("Circuit breaker open".to_string());
                }
            }
            CircuitState::HalfOpen => {
                if self.half_open_requests >= self.half_open_max_requests {
                    return Err("Circuit breaker open".to_string());
                }
                self.half_open_requests += 1;
            }
            CircuitState::Closed => {}
        }

        let result = operation();

        match result {
            Ok(value) => {
                if self.state == CircuitState::HalfOpen {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.half_open_requests = 0;
                }
                Ok(value)
            }
            Err(_) => {
                match self.state {
                    CircuitState::Closed => {
                        self.failure_count += 1;
                        if self.failure_count >= self.failure_threshold {
                            self.state = CircuitState::Open;
                            self.last_failure_time = Some(std::time::Instant::now());
                        }
                    }
                    CircuitState::HalfOpen => {
                        self.state = CircuitState::Open;
                        self.last_failure_time = Some(std::time::Instant::now());
                    }
                    _ => {}
                }
                Ok(T::default())
            }
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }

    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.last_failure_time = None;
        self.half_open_requests = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::MornError;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn retry_blocking_retries_until_success() {
        let attempts = AtomicUsize::new(0);
        let mut operation = |_| {
            let current = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if current < 3 {
                Err(MornError::Internal("not yet".to_string()))
            } else {
                Ok("ok")
            }
        };

        let result = retry_blocking_with_backoff(3, |_| Duration::from_millis(0), &mut operation);

        assert_eq!(result, Ok("ok"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_async_retries_until_success() {
        let attempts = AtomicUsize::new(0);

        let result = retry_async(|_| {
            let attempts = &attempts;
            async move {
                let current = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 2 {
                    Err(MornError::Internal("not yet".to_string()))
                } else {
                    Ok("ok")
                }
            }
        })
        .await;

        assert_eq!(result, Ok("ok"));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_circuit_breaker_closed_to_open() {
        let mut cb = CircuitBreaker::new(2, 1000, 1);
        let result1: Result<i32, String> = cb.call(|| Err("fail"));
        assert!(result1.is_ok());
        let _: Result<i32, String> = cb.call(|| Err("fail"));
        assert_eq!(*cb.state(), CircuitState::Open);
        let result3: Result<i32, String> = cb.call(|| -> Result<i32, String> { Ok(42) });
        assert!(result3.is_err());
    }

    #[test]
    fn test_circuit_breaker_half_open_recovery() {
        let mut cb = CircuitBreaker::new(1, 50, 1);
        let _ = cb.call::<i32, &str, _>(|| Err("fail"));
        assert_eq!(*cb.state(), CircuitState::Open);
        std::thread::sleep(std::time::Duration::from_millis(60));
        let result = cb.call(|| -> Result<i32, String> { Ok(42) });
        assert_eq!(*cb.state(), CircuitState::Closed);
        assert_eq!(result.unwrap_or(0), 42);
    }
}
