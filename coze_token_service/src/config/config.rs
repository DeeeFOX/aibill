use jsonwebtoken::EncodingKey;
use reqwest::Client;
use std::{env, sync::Arc};
use dotenvy::dotenv;
use crate::auth::model::AppConfig;

pub fn load_config() -> Arc<AppConfig> {
    dotenv().ok();

    let private_key_pem = env::var("JWT_PRIVATE_KEY").expect("JWT_PRIVATE_KEY must be set");
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).expect("Failed to create encoding key from PEM");

    let expected_coze_api_key = env::var("EXPECTED_COZE_API_KEY").expect("EXPECTED_COZE_API_KEY must be set");
    let coze_api_url = env::var("COZE_API_URL").expect("COZE_API_URL must be set");

    let http_client = Client::new();

    Arc::new(AppConfig {
        encoding_key,
        expected_coze_api_key,
        coze_api_url,
        http_client,
    })
}
