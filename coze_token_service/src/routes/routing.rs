use axum::Router;
use std::sync::Arc;
use crate::auth::model::AppConfig;
use crate::format::handler::resend_handler;
use crate::auth::handler::generate_and_exchange_token;

pub fn create_router() -> Router<Arc<AppConfig>> {
    Router::new()
    // 添加路由
    .route("/resend", axum::routing::post(resend_handler))
    .route("/token", axum::routing::post(generate_and_exchange_token))
}
