use askama::Template;
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::{markdown::find_post, models::post::Post, templates::HtmlTemplate};

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: Post,
}

pub async fn post_page(Path(slug): Path<String>) -> impl IntoResponse {
    match find_post(&slug) {
        Some(post) => HtmlTemplate(PostTemplate { post }).into_response(),
        None => (StatusCode::NOT_FOUND, "Post not found").into_response(),
    }
}
