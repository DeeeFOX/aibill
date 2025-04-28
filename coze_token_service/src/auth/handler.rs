use axum::{
    Json,
    extract::State,
    response::{Response, IntoResponse},
    http::StatusCode,
};
use jsonwebtoken::{encode, Algorithm, Header};
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use uuid::Uuid;
use tracing::{info, error, debug}; // Import tracing macros

use crate::auth::model::{AppConfig, TokenRequest, CozeTokenResponse, Claims, CozeTokenRequest};


pub async fn generate_and_exchange_token(
    State(config): State<Arc<AppConfig>>,
    Json(payload): Json<TokenRequest>,
) -> Result<Json<CozeTokenResponse>, Response> {
    info!("Received token generation and exchange request");
    debug!("Request payload: {:?}", payload);

    // --- API Key Validation ---
    if payload.coze_api_key != config.expected_coze_api_key {
        error!("Unauthorized attempt with invalid coze_api_key");
        return Err((StatusCode::UNAUTHORIZED, "Invalid coze_api_key").into_response());
    }
    info!("API key validated successfully");
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
        .map_err(|e| {
            error!("JWT encoding error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("JWT encoding error: {}", e)).into_response()
        })?;
    debug!("Generated JWT: {}", jwt_token);
    // --- End Generate JWT ---


    // --- Exchange JWT for Coze Access Token ---
    let coze_request_body = CozeTokenRequest {
        duration_seconds: duration, // Use the same duration
        grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer".to_string(),
    };
    debug!("Coze API request body: {:?}", coze_request_body);
    info!("Calling Coze API at {}", config.coze_api_url);

    let coze_response = config.http_client
        .post(&config.coze_api_url)
        .bearer_auth(&jwt_token)
        .json(&coze_request_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to call Coze API: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to call Coze API: {}", e)).into_response()
        })?;

    let status = coze_response.status();
    info!("Coze API responded with status: {}", status);

    if !status.is_success() {
        let error_body = coze_response.text().await.unwrap_or_else(|_| "Failed to read Coze error body".to_string());
        error!("Coze API returned error: {}", error_body);
        return Err((StatusCode::BAD_GATEWAY, format!("Coze API returned error: {}", error_body)).into_response());
    }

    let coze_token_response = coze_response.json::<CozeTokenResponse>()
        .await
        .map_err(|e| {
            error!("Failed to parse Coze API response: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse Coze API response: {}", e)).into_response()
        })?;
    debug!("Successfully received and parsed Coze API response");
    // --- End Exchange JWT ---

    // Return the response from Coze API
    info!("Token generation and exchange successful");
    Ok(Json(coze_token_response))
}
