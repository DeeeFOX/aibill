use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

#[derive(Debug)]
pub enum AppError {
    // 定义应用错误类型
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 错误处理逻辑
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
