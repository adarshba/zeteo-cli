use anyhow::Result;
use std::time::Duration;

pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, T, Fut>(
    operation: F,
    config: &RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                
                if attempt >= config.max_retries {
                    return Err(anyhow::anyhow!(
                        "Operation failed after {} attempts: {}",
                        config.max_retries,
                        e
                    ));
                }

                eprintln!(
                    "Attempt {}/{} failed: {}. Retrying in {:?}...",
                    attempt, config.max_retries, e, delay
                );

                tokio::time::sleep(delay).await;

                // Calculate next delay with exponential backoff
                delay = Duration::from_millis(
                    ((delay.as_millis() as f64) * config.multiplier) as u64
                ).min(config.max_delay);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_succeeds_on_first_attempt() {
        let config = RetryConfig::default();
        let call_count = Arc::new(AtomicU32::new(0));

        let result = retry_with_backoff(
            || {
                let count = call_count.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok::<i32, anyhow::Error>(42)
                }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
        };
        let call_count = Arc::new(AtomicU32::new(0));

        let result = retry_with_backoff(
            || {
                let count = call_count.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                    if current < 3 {
                        Err(anyhow::anyhow!("Temporary failure"))
                    } else {
                        Ok::<i32, anyhow::Error>(42)
                    }
                }
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
        };
        let call_count = Arc::new(AtomicU32::new(0));

        let result = retry_with_backoff(
            || {
                let count = call_count.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, anyhow::Error>(anyhow::anyhow!("Persistent failure"))
                }
            },
            &config,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
