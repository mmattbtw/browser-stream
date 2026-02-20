use std::time::Duration;

use browser_stream::retry::RetryPolicy;

#[test]
fn retries_up_to_configured_limit() {
    let policy = RetryPolicy::new(5, Duration::from_millis(100));

    assert!(policy.should_retry(1));
    assert!(policy.should_retry(5));
    assert!(!policy.should_retry(6));
}
