//! retry — Small shared retry helpers for network calls.

use std::future::Future;
use std::time::Duration;

pub const DEFAULT_RETRY_ATTEMPTS: usize = 3;
const DEFAULT_BACKOFF_MS: u64 = 200;

pub fn retry_blocking<T, E, F>(mut operation: F) -> Result<T, E>
where
    F: FnMut(usize) -> Result<T, E>,
{
    retry_blocking_with_backoff(DEFAULT_RETRY_ATTEMPTS, default_backoff, &mut operation)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn retry_blocking_retries_until_success() {
        let attempts = AtomicUsize::new(0);
        let mut operation = |_| {
            let current = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if current < 3 {
                Err("not yet")
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
                    Err("not yet")
                } else {
                    Ok("ok")
                }
            }
        })
        .await;

        assert_eq!(result, Ok("ok"));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }
}
