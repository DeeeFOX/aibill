// coze_token_service/tests/integration_tests.rs

use std::sync::Once;
use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio::task::JoinHandle;
use std::process::{Command, Child};
use std::io::BufReader;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

// Ensure the test server is only initialized once
static INIT: Once = Once::new();
static mut SERVER_PROCESS: Option<Child> = None;

fn setup() {
    INIT.call_once(|| {
        // Set environment variables for the test server
        // In a real scenario, you might use a test-specific .env file or generate keys
        std::env::set_var("JWT_PRIVATE_KEY", "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"); // Replace with a valid test key
        std::env::set_var("EXPECTED_COZE_API_KEY", "test_api_key");
        std::env::set_var("COZE_API_URL", "http://localhost:9001/mock_coze_token"); // Use a mock or test endpoint

        // Start the server in the background using cargo run
        let child = Command::new("cargo")
            .arg("run")
            .current_dir("../") // Assuming tests are run from coze_token_service/tests
            .spawn()
            .expect("failed to start server process");

        unsafe { SERVER_PROCESS = Some(child) };

        // Give the server a moment to start
        thread::sleep(Duration::from_secs(2)); // Increased sleep duration

        println!("Test setup complete");
    });
}

// Helper function to get the base URL of the test server
fn test_server_url() -> String {
    // This should match the address the test server is bound to
    "http://127.0.0.1:9000".to_string()
}

#[tokio::test]
async fn test_token_endpoint_success() {
    setup();

    let client = reqwest::Client::new();
    let url = format!("{}/token", test_server_url());

    let request_body = json!({
        "public_key": "test_key_id",
        "coze_api_key": "test_api_key",
        "duration_seconds": 3600
    });

    // TODO: Replace with a mock Coze API or ensure the mock endpoint is running
    // For now, this test will likely fail without a running mock at 9001
    let response = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Failed to parse JSON response");
    assert!(body.get("access_token").is_some());
    assert!(body.get("expires_in").is_some());
    assert!(body.get("token_type").is_some());

    println!("test_token_endpoint_success passed");
}

#[tokio::test]
async fn test_token_endpoint_unauthorized() {
    setup();

    let client = reqwest::Client::new();
    let url = format!("{}/token", test_server_url());

    let request_body = json!({
        "public_key": "test_key_id",
        "coze_api_key": "wrong_api_key", // Invalid API key
        "duration_seconds": 3600
    });

    let response = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    println!("test_token_endpoint_unauthorized passed");
}

#[tokio::test]
async fn test_token_endpoint_bad_request() {
    setup();

    let client = reqwest::Client::new();
    let url = format!("{}/token", test_server_url());

    // Missing required field (coze_api_key)
    let request_body = json!({
        "public_key": "test_key_id",
        // "coze_api_key": "test_api_key",
        "duration_seconds": 3600
    });

    let response = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    println!("test_token_endpoint_bad_request passed");
}

#[tokio::test]
async fn test_resend_endpoint_success() {
    setup();

    let client = reqwest::Client::new();
    let url = format!("{}/resend", test_server_url());

    // This test requires a mock endpoint that the /resend endpoint can successfully call.
    // For demonstration, we'll use a placeholder URL.
    // In a real test, you would set up a mock server here.
    let request_body = json!({
        "location": "http://localhost:9001/mock_resend_target", // Replace with a mock target URL
        "headers": {
            "X-Test-Header": "test_value"
        },
        "params": {
            "test_param": "param_value"
        },
        "commands": {
            "json_parse": ["$.some_field"]
        }
    });

    // TODO: Replace with a mock target or ensure the mock endpoint is running at 9001
    let response = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    // This assertion depends on the mock target's expected response
    // For a successful resend, the service should return the response from the location URL
    assert_eq!(response.status(), StatusCode::OK);

    // Further assertions on the response body would depend on the mock target's output
    println!("test_resend_endpoint_success passed (requires mock target)");
}

#[tokio::test]
async fn test_resend_endpoint_bad_request() {
    setup();

    let client = reqwest::Client::new();
    let url = format!("{}/resend", test_server_url());

    // Missing required field (location)
    let request_body = json!({
        // "location": "http://localhost:9001/mock_resend_target",
        "headers": {
            "X-Test-Header": "test_value"
        }
    });

    let response = client.post(&url)
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    println!("test_resend_endpoint_bad_request passed");
}

// Add more /resend tests for different scenarios (e.g., invalid URL, target returns error)

// Note: Properly shutting down the background server process after tests
// is complex with this simple setup. In a more robust test framework,
// you would manage the server lifecycle more explicitly.
