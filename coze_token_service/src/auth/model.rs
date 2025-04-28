use serde::{Deserialize, Serialize};
use jsonwebtoken::EncodingKey;
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleModel {
    pub id: i32,
    pub name: String,
    pub description: String,
}

// AppConfig holds JWT key, expected API key, and Coze API URL
#[derive(Clone)]
pub struct AppConfig {
    pub encoding_key: EncodingKey,
    pub expected_coze_api_key: String,
    pub coze_api_url: String, // Added Coze API URL
    pub http_client: Client, // Added HTTP client
}

// Request body for our service
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub public_key: String,
    pub coze_api_key: String,
    pub duration_seconds: Option<u64>,
}

// Request body for Coze API
#[derive(Debug, Serialize)]
pub struct CozeTokenRequest {
    pub duration_seconds: u64,
    pub grant_type: String,
}

// Response body from Coze API (and our service)
#[derive(Deserialize, Serialize, Debug)]
pub struct CozeTokenResponse {
    pub access_token: String,
    pub expires_in: i64, // Coze API might return seconds, adjust if needed
    pub token_type: String,
}

// Claims structure for the JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
    pub aud: String,
    pub iss: String,
}
