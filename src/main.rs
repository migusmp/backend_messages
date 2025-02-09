use std::sync::Arc;

use axum::{routing::get, Router};
use backend_messages::{
    db::db::init_db_pool, models::chat::ChatState, routes::main_router::main_router,
    utils::cors::create_cors_layer,
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let pool = init_db_pool().await;
    let cors = create_cors_layer();
    let chat_state = Arc::new(RwLock::new(ChatState::default()));

    let app = Router::new()
        .nest("/api", main_router(pool, chat_state))
        .route("/test", get(|| async { "Welcome to message microservice" }))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
