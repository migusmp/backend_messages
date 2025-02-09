use super::{chat::chat_router, friend::friend_router};
use crate::models::chat::ChatState;
use axum::Router;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn main_router(pool: Arc<Pool<Postgres>>, chat_state: Arc<RwLock<ChatState>>) -> Router {
    let chat_router = chat_router(chat_state.clone(), pool.clone());
    let friend_router = friend_router(pool.clone());

    Router::new()
        .nest("/chat", chat_router)
        .nest("/friend", friend_router)
}
