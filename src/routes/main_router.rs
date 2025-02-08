use super::{chat::chat_router, friend::friend_router};
use crate::{
    controller::ws_controller::handle_ws_connection, middlewares::auth::auth,
    models::chat::ChatState, state::app_state::AppState,
};
use axum::{middleware::from_fn, routing::get, Router};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn main_router(
    pool: Arc<Pool<Postgres>>,
    chat_state: Arc<RwLock<ChatState>>,
    app_state: Arc<AppState>,
) -> Router {
    let chat_router = chat_router(chat_state.clone(), pool.clone());
    let friend_router = friend_router(pool.clone(), app_state.clone());

    Router::new()
        .route(
            "/ws",
            get({
                let app_state = app_state.clone();
                move |payload, ws| handle_ws_connection(ws, app_state, payload)
            })
            .route_layer(from_fn(auth)),
        )
        .nest("/chat", chat_router)
        .nest("/friend", friend_router)
}
