use std::{borrow::Cow, sync::Arc};

use axum::{http::StatusCode, response::Response};
use bcrypt::BcryptError;
use chrono::{Duration, Utc};
use cookie::Cookie;
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use sqlx::PgPool;
use tokio::task::JoinError;

use crate::models::user::{LoginUser, Payload, RegisterUser, User};

// Verificamos que el usuario exista
pub async fn verify_user_exists(
    user: &RegisterUser,
    pool: &Arc<PgPool>,
) -> Result<bool, JoinError> {
    // Consulta SQL para verificar si el usuario existe por correo o nombre de usuario
    let result: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) 
        FROM users 
        WHERE email = $1 OR username = $2
        "#,
    )
    .bind(&user.email)
    .bind(&user.username)
    .fetch_one(&**pool) // Ejecuta la consulta de forma asíncrona
    .await
    .unwrap();

    Ok(result.0 > 0) // Si COUNT(*) > 0, el usuario existe
}

// Función para verificar que el usuario y la contraseña son correctos
pub async fn verify_user_login(
    user_login: &LoginUser,
    pool: &PgPool,
) -> Result<Option<User>, sqlx::Error> {
    // Realizamos la consulta para obtener los datos del usuario por `username`.
    let row = sqlx::query!(
        r#"
        SELECT id, username, email, password, image, created_at
        FROM users
        WHERE username = $1
        "#,
        user_login.username
    )
    .fetch_optional(pool)
    .await?;

    // Si no se encuentra el usuario, devolvemos `None`.
    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };

    if hashed_pwd(&user_login.password, &row.password).unwrap() {
        let created_at = row.created_at.map(|dt| dt.to_string());

        Ok(Some(User {
            id: row.id,
            name: row.username,
            email: row.email,
            image: row.image.unwrap(),
            password: row.password,
            created_at,
        }))
    } else {
        Ok(None)
    }
}

pub fn hashed_pwd(pwd: &String, hashed_pwd: &str) -> Result<bool, BcryptError> {
    bcrypt::verify(pwd, hashed_pwd)
}

pub async fn create_payload(user_data: User) -> Result<String, jsonwebtoken::errors::Error> {
    let iat = Utc::now().timestamp(); // tiempo actual
    let exp = (Utc::now() + Duration::hours(1)).timestamp(); // 1 hora de tiempo de expiración.
    let user_payload = Payload::new(
        user_data.id,
        user_data.name,
        user_data.email,
        user_data.image,
        user_data.password,
        user_data.created_at,
        exp,
        iat,
    );
    user_payload.token()
}

pub async fn create_token_cookie<'a>(name_cookie: &'a str, token: Cow<'a, str>) -> Cookie<'a> {
    Cookie::build((name_cookie, token))
        .http_only(true) // Evita que sea accesible desde JavaScript.
        // .same_site(cookie::SameSite::Lax)
        .same_site(cookie::SameSite::None)
        // .secure(false)
        .secure(true)
        .partitioned(true)
        .path("/")
        .build()
}

pub fn append_cookie_to_response(res: &mut Response, cookie: Cookie) {
    let cookie_header = cookie.to_string();
    res.headers_mut().append(
        axum::http::header::SET_COOKIE,
        cookie_header.parse().unwrap(),
    );
}

pub async fn decode_token(auth_token: String) -> Result<Payload, StatusCode> {
    let token_data: TokenData<Payload> = jsonwebtoken::decode(
        &auth_token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        eprintln!("Error al decodificar el token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(token_data.claims)
}
