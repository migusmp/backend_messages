use std::sync::Arc;

use axum::{routing::post, Router};
use sqlx::PgPool;

use crate::{
    controller::friend_controller::{accept_friend_request, send_friend_request},
    middlewares::auth::auth,
};

pub fn friend_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/add/{friend_id}",
            post({
                let pool_for_friend_add = pool.clone();
                move |payload, path_data| {
                    send_friend_request(pool_for_friend_add, payload, path_data)
                }
            }),
        )
        .route(
            "/accept/{user_requested_friend_id}",
            post({
                let pool = pool.clone();
                move |payload, path_data| accept_friend_request(pool, payload, path_data)
            }),
        )
        .layer(axum::middleware::from_fn(auth))
}
