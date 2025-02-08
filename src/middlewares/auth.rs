use crate::errors::ErrorAuth;
use crate::utils::{responses::ApiResponse, user_utils::decode_token};
use axum::{
    body::Body,
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

pub async fn auth(mut req: Request<Body>, next: Next) -> impl IntoResponse {
    // Obtenemos todos los headers de la petición.
    let headers = req.headers().clone();
    // Si esta el header 'cookie' ejecutamos el siguiente codigo
    if let Some(cookie_header) = headers.get("cookie") {
        match get_auth_cookie(cookie_header).await {
            Ok(auth_token) => {
                match decode_token(auth_token.to_string()).await {
                    Ok(payload) => {
                        // let mut req = req.map(|_body| Body::empty());
                        // Insertamos el payload como extension en los headers.
                        req.extensions_mut().insert(payload);
                        return next.run(req).await;
                    }
                    Err(_) => {
                        return (ApiResponse::error(StatusCode::UNAUTHORIZED, "Unauthorized"))
                            .into_response();
                    }
                };
            }
            Err(e) => match e {
                ErrorAuth::NoneCookieFound => {
                    (ApiResponse::error(StatusCode::UNAUTHORIZED, "Autentication cookie not found"))
                        .into_response()
                }
                ErrorAuth::AuthCookieNotFound => {
                    (ApiResponse::error(StatusCode::UNAUTHORIZED, "None cookies found in request"))
                        .into_response()
                }
            },
        };
    }
    (ApiResponse::error(StatusCode::UNAUTHORIZED, "Unauthorized")).into_response()
}

async fn get_auth_cookie(cookie_header: &HeaderValue) -> Result<String, ErrorAuth> {
    if let Ok(cookie_str) = cookie_header.to_str() {
        for cookie in cookie_str.split(";") {
            if let Some(auth_token) = cookie.strip_prefix("auth=") {
                return Ok(auth_token.to_string());
            } else {
                return Err(ErrorAuth::AuthCookieNotFound);
            }
        }
    }
    return Err(ErrorAuth::NoneCookieFound);
}
