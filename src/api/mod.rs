use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    api::routes::{handle_get_users, handle_post_user, handle_root},
    storage::Store,
};

mod routes;

pub fn create_router(store: Store) -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/users", get(handle_get_users))
        .route("/user", post(handle_post_user))
        .with_state(store)
}
