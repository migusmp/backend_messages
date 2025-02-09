use crate::{
    models::user::{ErrorRequest, Payload},
    state::app_state::AppState,
    utils::responses::ApiResponse,
};
use axum::{extract::Path, response::IntoResponse, Extension};
use sqlx::PgPool;
use std::sync::Arc;

pub async fn send_friend_request(
    pool: Arc<PgPool>,
    Extension(payload): Extension<Payload>,
    Path(friend_id): Path<String>,
) -> Result<impl IntoResponse, ErrorRequest> {
    // Intentar parsear el ID del amigo
    let friend_id = friend_id
        .parse::<i32>()
        .map_err(|_| ErrorRequest::InvalidFriendRequest)?;

    // Asegurarse de que el ID del amigo no sea el mismo que el del usuario
    if payload.id == friend_id {
        return Err(ErrorRequest::InvalidFriendRequest);
    }

    // Verificar si ya existe una solicitud de amistad pendiente
    let existing_request = sqlx::query_scalar!(
        "SELECT 1 FROM friend_requests WHERE user_id = $1 AND friend_id = $2 AND status = 'pending'",
        payload.id,
        friend_id
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|_| ErrorRequest::InternalError)?;

    if existing_request.is_some() {
        return Err(ErrorRequest::DuplicateFriendRequest);
    }

    // Insertar la solicitud de amistad en la base de datos
    sqlx::query!(
        "INSERT INTO friend_requests (user_id, friend_id, status, created_at) VALUES ($1, $2, 'pending', NOW())",
        payload.id,
        friend_id,
    )
    .execute(&*pool)
    .await
    .map_err(|_| ErrorRequest::InternalError)?;

    // Enviar notificación de solicitud de amistad
    AppState::send_friend_notification_to_kafka(friend_id, payload.name, payload.id).await;

    Ok(ApiResponse::success("User friend requested successfully"))
}

pub async fn accept_friend_request(
    pool: Arc<PgPool>,
    Extension(payload): Extension<Payload>,
    Path(user_requested_friend_id): Path<String>,
) -> Result<impl IntoResponse, ErrorRequest> {
    let friend_requested_id = user_requested_friend_id
        .parse::<i32>()
        .map_err(|_| ErrorRequest::InternalError)?;

    if payload.id == friend_requested_id {
        return Err(ErrorRequest::InvalidFriendRequest);
    }

    // Verificar si la solicitud de amistad existe y está pendiente
    let existing_request = sqlx::query_scalar!(
        "SELECT 1 FROM friend_requests WHERE user_id = $1 AND friend_id = $2 AND status = 'pending'",
        friend_requested_id,
        payload.id
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|_| ErrorRequest::InternalError)?;

    if existing_request.is_none() {
        return Err(ErrorRequest::NoFriendRequestFound);
    }

    // Actualizar la solicitud de amistad a 'accepted'
    sqlx::query!(
        "UPDATE friend_requests SET status = 'accepted' WHERE user_id = $1 AND friend_id = $2",
        friend_requested_id,
        payload.id
    )
    .execute(&*pool)
    .await
    .map_err(|_| ErrorRequest::InternalError)?;

    // Insertar la relación de amistad en la tabla `friends`
    sqlx::query!(
        "INSERT INTO friends (user_id, friend_id) VALUES ($1, $2), ($2, $1)",
        friend_requested_id,
        payload.id
    )
    .execute(&*pool)
    .await
    .map_err(|_| ErrorRequest::InternalError)?;

    // Enviar notificación de aceptación de solicitud de amistad
    AppState::accept_friend_notification_kafka(friend_requested_id, payload.name, payload.id).await;

    // Responder exitosamente
    Ok(ApiResponse::success("Friend added successfully"))
}
