// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::metrics;
use reqwest::StatusCode;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error as MiddlewareError};
use reqwest_retry::{
    RetryDecision, RetryPolicy, RetryTransientMiddleware, Retryable, RetryableStrategy,
    policies::ExponentialBackoff,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

pub type HttpClient = ClientWithMiddleware;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub connect_timeout_ms: u64,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub min_retry_delay_ms: u64,
    pub max_retry_delay_ms: u64,
    pub restrict_to_public: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 1_500,
            timeout_ms: 30_000,
            max_retries: 2,
            min_retry_delay_ms: 25,
            max_retry_delay_ms: 500,
            restrict_to_public: false,
        }
    }
}

pub fn build_raw(options: Options) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(options.connect_timeout_ms.max(1)))
        .timeout(Duration::from_millis(options.timeout_ms.max(1)))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(crate::constants::OUTBOUND_USER_AGENT);
    if options.restrict_to_public {
        builder = builder.dns_resolver(std::sync::Arc::new(
            crate::public_net_policy::PinnedDnsResolver,
        ));
    }
    builder.build()
}

pub fn build(options: Options) -> Result<HttpClient, reqwest::Error> {
    let client = build_raw(options)?;
    let builder = ClientBuilder::new(client);
    if options.max_retries == 0 {
        return Ok(builder.build());
    }
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(
            Duration::from_millis(options.min_retry_delay_ms.max(1)),
            Duration::from_millis(
                options
                    .max_retry_delay_ms
                    .max(options.min_retry_delay_ms)
                    .max(1),
            ),
        )
        .build_with_max_retries(options.max_retries);
    Ok(builder
        .with(RetryTransientMiddleware::new_with_policy_and_strategy(
            ObservableRetryPolicy {
                inner: retry_policy,
            },
            MediaProxyRetryStrategy,
        ))
        .build())
}

pub fn build_default() -> HttpClient {
    build(Options::default()).expect("default HTTP client configuration is valid")
}

pub fn build_raw_default() -> reqwest::Client {
    build_raw(Options::default()).expect("default HTTP client configuration is valid")
}

struct ObservableRetryPolicy {
    inner: ExponentialBackoff,
}

impl RetryPolicy for ObservableRetryPolicy {
    fn should_retry(&self, request_start_time: SystemTime, n_past_retries: u32) -> RetryDecision {
        let decision = self.inner.should_retry(request_start_time, n_past_retries);
        match decision {
            RetryDecision::Retry { .. } => {
                metrics::GLOBAL.http_retries.fetch_add(1, Ordering::Relaxed);
            }
            RetryDecision::DoNotRetry => {
                metrics::GLOBAL
                    .http_retries_exhausted
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
        decision
    }
}

struct MediaProxyRetryStrategy;

impl RetryableStrategy for MediaProxyRetryStrategy {
    fn handle(&self, result: &Result<reqwest::Response, MiddlewareError>) -> Option<Retryable> {
        match result {
            Ok(response) => retryable_status(response.status()).map(|retryable| {
                if retryable == Retryable::Transient {
                    metrics::GLOBAL
                        .http_retryable_status
                        .fetch_add(1, Ordering::Relaxed);
                }
                retryable
            }),
            Err(error) => retryable_error(error).map(|retryable| {
                if retryable == Retryable::Transient {
                    metrics::GLOBAL
                        .http_retryable_error
                        .fetch_add(1, Ordering::Relaxed);
                }
                retryable
            }),
        }
    }
}

fn retryable_status(status: StatusCode) -> Option<Retryable> {
    match status {
        StatusCode::REQUEST_TIMEOUT
        | StatusCode::TOO_MANY_REQUESTS
        | StatusCode::INTERNAL_SERVER_ERROR
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => Some(Retryable::Transient),
        status if status.is_client_error() || status.is_server_error() => Some(Retryable::Fatal),
        _ => None,
    }
}

fn retryable_error(error: &MiddlewareError) -> Option<Retryable> {
    match error {
        MiddlewareError::Middleware(_) => Some(Retryable::Fatal),
        MiddlewareError::Reqwest(error) => {
            #[cfg(not(target_arch = "wasm32"))]
            let is_connect = error.is_connect();
            #[cfg(target_arch = "wasm32")]
            let is_connect = false;

            if error.is_timeout() || is_connect {
                Some(Retryable::Transient)
            } else if error.is_body()
                || error.is_decode()
                || error.is_builder()
                || error.is_redirect()
                || error.is_status()
            {
                Some(Retryable::Fatal)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_retrying_client() {
        let client = build(Options {
            connect_timeout_ms: 0,
            timeout_ms: 0,
            max_retries: 1,
            min_retry_delay_ms: 0,
            max_retry_delay_ms: 0,
            restrict_to_public: false,
        });
        assert!(client.is_ok());
    }

    #[test]
    fn builds_non_retrying_client() {
        let client = build(Options {
            max_retries: 0,
            ..Options::default()
        });
        assert!(client.is_ok());
    }

    #[test]
    fn retry_strategy_retries_only_explicit_transient_statuses() {
        assert!(matches!(
            retryable_status(StatusCode::REQUEST_TIMEOUT),
            Some(Retryable::Transient)
        ));
        assert!(matches!(
            retryable_status(StatusCode::TOO_MANY_REQUESTS),
            Some(Retryable::Transient)
        ));
        assert!(matches!(
            retryable_status(StatusCode::INTERNAL_SERVER_ERROR),
            Some(Retryable::Transient)
        ));
        assert!(matches!(
            retryable_status(StatusCode::BAD_GATEWAY),
            Some(Retryable::Transient)
        ));
        assert!(matches!(
            retryable_status(StatusCode::SERVICE_UNAVAILABLE),
            Some(Retryable::Transient)
        ));
        assert!(matches!(
            retryable_status(StatusCode::GATEWAY_TIMEOUT),
            Some(Retryable::Transient)
        ));
        assert!(matches!(
            retryable_status(StatusCode::NOT_FOUND),
            Some(Retryable::Fatal)
        ));
        assert!(matches!(
            retryable_status(StatusCode::NOT_IMPLEMENTED),
            Some(Retryable::Fatal)
        ));
        assert!(retryable_status(StatusCode::OK).is_none());
        assert!(retryable_status(StatusCode::FOUND).is_none());
    }
}
