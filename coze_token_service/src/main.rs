use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
    response::{Response, IntoResponse},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Algorithm, Header, EncodingKey};
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}, env};
use uuid::Uuid;
use dotenvy::dotenv;
use reqwest::Client; // Import reqwest client

// AppConfig holds JWT key, expected API key, and Coze API URL
#[derive(Clone)]
struct AppConfig {
    encoding_key: EncodingKey,
    expected_coze_api_key: String,
    coze_api_url: String, // Added Coze API URL
    http_client: Client, // Added HTTP client
}

// Request body for our service
#[derive(Deserialize)]
struct TokenRequest {
    public_key: String,
    coze_api_key: String,
    duration_seconds: Option<u64>,
}

// Request body for Coze API
#[derive(Serialize)]
struct CozeTokenRequest {
    duration_seconds: u64,
    grant_type: String,
}

// Response body from Coze API (and our service)
#[derive(Deserialize, Serialize, Debug)]
struct CozeTokenResponse {
    access_token: String,
    expires_in: i64, // Coze API might return seconds, adjust if needed
    token_type: String,
}

// Claims structure for the JWT
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: i64,
    exp: i64,
    jti: String,
    aud: String,
    iss: String,
}

async fn generate_and_exchange_token(
    State(config): State<Arc<AppConfig>>,
    Json(payload): Json<TokenRequest>,
) -> Result<Json<CozeTokenResponse>, Response> {

    // --- API Key Validation ---
    if payload.coze_api_key != config.expected_coze_api_key {
        return Err((StatusCode::UNAUTHORIZED, "Invalid coze_api_key").into_response());
    }
    // --- End API Key Validation ---

    // --- Generate JWT ---
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(payload.public_key.clone());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("System time error: {}", e)).into_response())?
        .as_secs() as i64;

    // Default duration: 24 hours (86400 seconds)
    let duration = payload.duration_seconds.unwrap_or(86400);
    let exp = now + duration as i64;

    let claims = Claims {
        iat: now,
        exp,
        jti: Uuid::new_v4().to_string(),
        aud: "api.coze.cn".to_string(), // Keep audience as required by Coze
        iss: payload.coze_api_key.clone(),
    };

    let jwt_token = encode(&header, &claims, &config.encoding_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JWT encoding error: {}", e)).into_response())?;
    // --- End Generate JWT ---


    // --- Exchange JWT for Coze Access Token ---
    let coze_request_body = CozeTokenRequest {
        duration_seconds: duration, // Use the same duration
        grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer".to_string(),
    };

    let coze_response = config.http_client
        .post(&config.coze_api_url)
        .bearer_auth(&jwt_token)
        .json(&coze_request_body)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to call Coze API: {}", e)).into_response())?;

    if !coze_response.status().is_success() {
        let error_body = coze_response.text().await.unwrap_or_else(|_| "Failed to read Coze error body".to_string());
        return Err((StatusCode::BAD_GATEWAY, format!("Coze API returned error: {}", error_body)).into_response());
    }

    let coze_token_response = coze_response.json::<CozeTokenResponse>()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse Coze API response: {}", e)).into_response())?;
    // --- End Exchange JWT ---

    // Return the response from Coze API
    Ok(Json(coze_token_response))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let private_key_pem = env::var("JWT_PRIVATE_KEY")
        .expect("JWT_PRIVATE_KEY environment variable must be set");
    let expected_api_key = env::var("EXPECTED_COZE_API_KEY")
        .expect("EXPECTED_COZE_API_KEY environment variable must be set");
    let coze_api_url = env::var("COZE_API_URL")
        .expect("COZE_API_URL environment variable must be set");

    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.trim().as_bytes())
        .expect("Failed to load RSA private key from PEM");

    // Create a single reqwest client to be reused
    let http_client = Client::new();

    let config = Arc::new(AppConfig {
        encoding_key,
        expected_coze_api_key: expected_api_key,
        coze_api_url,
        http_client,
    });

    let app = Router::new()
        // Update route handler name
        .route("/token", post(generate_and_exchange_token))
        .with_state(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
