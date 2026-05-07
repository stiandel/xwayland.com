pub mod blog;
pub mod about;
pub mod cv;

use axum::Router;
use tower_http::services::ServeDir;

/// Assembles the full application router.
/// Add new routers here as the site grows — e.g. api::router(), auth::router().
pub fn app() -> Router {
    Router::new()
        .merge(blog::router())
        .merge(about::router())
        .merge(cv::router())
        // Static assets
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/public", ServeDir::new("public"))
}
