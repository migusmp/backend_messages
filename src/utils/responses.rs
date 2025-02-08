use axum::{http::StatusCode, response::IntoResponse, Json};
use cookie::Cookie;
use serde::Serialize;
use serde_json::json;

use super::user_utils::append_cookie_to_response;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

pub enum ApiResponse<'a> {
    Success(StatusCode, Json<serde_json::Value>),
    SuccessWithData(StatusCode, Json<serde_json::Value>),
    SuccessWithCookie(StatusCode, Json<serde_json::Value>, Cookie<'a>),
    Error(StatusCode, Json<serde_json::Value>),
}

impl<'a> IntoResponse for ApiResponse<'a> {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiResponse::Success(status, msg) => (status, msg).into_response(),
            ApiResponse::SuccessWithData(status, msg) => (status, msg).into_response(),
            ApiResponse::SuccessWithCookie(status, msg, cookie) => {
                let mut response = (status, msg).into_response();
                append_cookie_to_response(&mut response, cookie);
                response
            }
            ApiResponse::Error(status, msg) => (status, msg).into_response(),
        }
    }
}

impl<'a> ApiResponse<'a> {
    pub fn success(message: &str) -> Self {
        let success_msg = json!({
            "status": "success",
            "message": message,
        });
        ApiResponse::Success(StatusCode::OK, Json(success_msg))
    }

    pub fn success_with_data<T: Serialize>(message: &str, data: Option<T>) -> Self {
        let mut success_msg = json!({
            "status": "success",
            "message": message,
        });
        if let Some(data) = data {
            success_msg["data"] = json!(data);
        }
        ApiResponse::SuccessWithData(StatusCode::OK, Json(success_msg))
    }

    pub fn success_with_cookie(msg: &'a str, cookie: Cookie<'a>) -> Self {
        let success_msg = json!({
            "status": "success",
            "message": msg,
        });
        ApiResponse::SuccessWithCookie(StatusCode::OK, Json(success_msg), cookie)
    }

    pub fn error(status: StatusCode, message: &str) -> Self {
        let err_msg = json!({
            "status": "error",
            "message": message,
        });
        ApiResponse::Error(status, Json(err_msg))
    }
}
