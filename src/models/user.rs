use crate::utils::jwt::generate_token;
use crate::utils::responses::ErrorResponse;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct RegisterUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub image: String,
    pub password: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payload {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub image: String,
    pub password: String,
    pub created_at: Option<String>,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateData {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

pub enum ErrorRequest {
    UsernameInvalid,
    UsernameEmpty,
    EmailInvalid,
    PasswordInvalid,
    UserAlreadyExists,
    InternalError,
    InvalidFriendRequest,
    DuplicateFriendRequest,
    NoFriendRequestFound,
    InvalidImageSize,
    InvalidImageFormat,
}

impl IntoResponse for ErrorRequest {
    fn into_response(self) -> axum::response::Response {
        let (status, err_msg) = match self {
            ErrorRequest::UsernameInvalid => (StatusCode::BAD_REQUEST, "Invalid username"),
            ErrorRequest::UsernameEmpty => (StatusCode::BAD_REQUEST, "You must enter a username"),
            ErrorRequest::EmailInvalid => (StatusCode::BAD_REQUEST, "Invalid email"),
            ErrorRequest::PasswordInvalid => (StatusCode::BAD_REQUEST, "invalid password"),
            ErrorRequest::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            ErrorRequest::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
            ErrorRequest::InvalidImageSize => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Image exceeds the maximum allowed size of 5MB.",
            ),
            ErrorRequest::InvalidFriendRequest => (
                StatusCode::BAD_REQUEST,
                "You can't add yourself as a friend",
            ),
            ErrorRequest::DuplicateFriendRequest => {
                (StatusCode::OK, "You requested frienship before")
            }

            ErrorRequest::NoFriendRequestFound => {
                (StatusCode::BAD_REQUEST, "No friend request found")
            }

            ErrorRequest::InvalidImageFormat => (StatusCode::BAD_REQUEST, "Invalid Image format"),
        };
        let body = Json(ErrorResponse {
            status: "error".to_string(),
            message: err_msg.to_string(),
        });
        (status, body).into_response()
    }
}

impl RegisterUser {
    pub fn new(username: String, email: String, password: String) -> RegisterUser {
        RegisterUser {
            username,
            email,
            password,
        }
    }
}

impl LoginUser {
    pub fn new(username: String, password: String) -> Self {
        LoginUser { username, password }
    }
}

impl Payload {
    pub fn new(
        id: i32,
        name: String,
        email: String,
        password: String,
        image: String,
        created_at: Option<String>,
        exp: i64,
        iat: i64,
    ) -> Self {
        let created_at = match created_at {
            Some(datetime) => Some(datetime.to_string()),
            None => None,
        };

        Payload {
            id,
            name,
            email,
            image,
            password,
            created_at,
            exp,
            iat,
        }
    }

    pub fn token(&self) -> Result<String, jsonwebtoken::errors::Error> {
        generate_token(self)
    }
}
