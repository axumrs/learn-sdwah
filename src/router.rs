use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::services::ServeDir;

use crate::ch01_jumping_in;

pub fn init() -> Router {
    Router::new()
        .nest("/api", api())
        .fallback_service(ServeDir::new("htmx_files"))
}

fn api() -> Router {
    Router::new().nest("/ch01", ch01())
}

fn ch01() -> Router {
    Router::new()
        .route("/version", get(ch01_jumping_in::version))
        .route("/dog", post(ch01_jumping_in::add_dog))
        .route("/table-rows", get(ch01_jumping_in::dog_rows))
        .route("/dog/{id}", delete(ch01_jumping_in::del))
}
