use axum::{routing::get, Router};

use crate::handlers::{home::home, posts::post_page};

pub fn router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/posts/{slug}", get(post_page))
}
