#![cfg(feature = "test-mock")]

mod mock;

use std::time::Duration;

use jpf4826_driver::{Jpf4826Client, DEFAULT_TIMEOUT};
use mock::MockController;

async fn create_test_client() -> (Jpf4826Client, MockController) {
    let mock = MockController::new();
    let registers = mock.registers.clone();
    let client = Jpf4826Client::new_mock(registers, 1).await;
    (client, mock)
}

#[test]
fn test_default_timeout_is_ten_seconds() {
    assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(10));
}

#[tokio::test]
async fn test_mock_client_returns_default_timeout() {
    let (client, _mock) = create_test_client().await;

    assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
}

#[tokio::test]
async fn test_set_timeout_on_mock_client_is_noop() {
    let (mut client, _mock) = create_test_client().await;

    // Mock backend ignores set_timeout; timeout remains DEFAULT_TIMEOUT
    client.set_timeout(Duration::from_secs(30));
    assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
}
