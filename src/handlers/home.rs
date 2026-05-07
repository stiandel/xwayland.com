use askama::Template;
use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

use crate::{markdown::load_all_posts, models::post::Post, templates::HtmlTemplate};

const POSTS_PER_PAGE: usize = 6;

#[derive(Deserialize)]
pub struct Pagination {
    pub page: Option<i32>,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    featured: Post,
    posts: Vec<Post>,
    page: i32,
    total_pages: i32,
}

pub async fn home(Query(params): Query<Pagination>) -> impl IntoResponse {
    let all_posts = load_all_posts();

    if all_posts.is_empty() {
        return axum::response::Html("<p>No posts yet.</p>".to_string()).into_response();
    }

    let current_page = params.page.unwrap_or(1).max(1) as usize;
    let featured = all_posts[0].clone();

    let rest: Vec<Post> = all_posts.into_iter().skip(1).collect();
    let total_pages = ((rest.len() as f64) / POSTS_PER_PAGE as f64).ceil() as usize;
    let start = (current_page - 1) * POSTS_PER_PAGE;
    let posts: Vec<Post> = rest.into_iter().skip(start).take(POSTS_PER_PAGE).collect();

    HtmlTemplate(HomeTemplate {
        featured,
        posts,
        page: current_page as i32,
        total_pages: total_pages.max(1) as i32,
    })
    .into_response()
}
