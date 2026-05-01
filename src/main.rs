use std::{fs, path::Path};

use askama::Template;
use axum::{
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use gray_matter::{engine::YAML, Matter, ParsedEntity};
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use tower_http::services::ServeDir;

// ── Post model ────────────────────────────────────────────────────────────────

/// Matches the YAML frontmatter fields in each .md file under content/
#[derive(Debug, Clone, Deserialize)]
struct Frontmatter {
    title: String,
    date: String,
    excerpt: String,
    tags: Vec<String>,
    read_time: i32,
    image: String,
}

#[derive(Debug, Clone)]
struct Post {
    slug: String,
    title: String,
    date: String,         // "2025-05-01"  — used for sorting
    date_display: String, // "May 1, 2025" — rendered in templates
    date_short: String,   // "May 1"       — rendered on cards
    excerpt: String,
    // Field is called `content` so post.html can use {{ post.content|safe }}
    content: String,
    tags: Vec<String>,
    read_time: i32,
    image: String,
}

impl Post {
    fn primary_tag(&self) -> &str {
        self.tags.first().map(|s| s.as_str()).unwrap_or("general")
    }
}

// ── Markdown loading ──────────────────────────────────────────────────────────

/// Parse a YYYY-MM-DD date string into display forms.
fn format_date(date: &str) -> (String, String) {
    let months = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(m), Ok(d), Ok(y)) = (
            parts[1].parse::<usize>(),
            parts[2].parse::<u32>(),
            parts[0].parse::<u32>(),
        ) {
            if (1..=12).contains(&m) {
                let month = months[m - 1];
                return (
                    format!("{} {}, {}", month, d, y),
                    format!("{} {}", month, d),
                );
            }
        }
    }
    (date.to_string(), date.to_string())
}

/// Convert Markdown text to an HTML string.
fn markdown_to_html(md: &str) -> String {
    let opts = Options::all();
    let parser = Parser::new_ext(md, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

/// Load and parse a single .md file from `content/`.
/// Returns None if the file is missing or malformed.
fn load_post(path: &Path) -> Option<Post> {
    let raw = fs::read_to_string(path).ok()?;

    let matter = Matter::<YAML>::new();
    // parse() returns Result<ParsedEntity, Error>
    let parsed: ParsedEntity<gray_matter::Pod> = matter.parse(&raw).ok()?;

    // .data is Option<Pod> — deserialize into our typed Frontmatter struct
    let frontmatter: Frontmatter = parsed.data?.deserialize().ok()?;
    // .content is the body text after the --- block
    let content = markdown_to_html(&parsed.content);

    // Derive slug from filename: "2025-05-01-my-post.md" → "my-post"
    let filename = path.file_stem()?.to_string_lossy();
    // Strip leading YYYY-MM-DD- prefix (11 chars) if present
    let slug = if filename.len() > 11 && filename.chars().nth(4) == Some('-') {
        filename[11..].to_string()
    } else {
        filename.to_string()
    };

    let (date_display, date_short) = format_date(&frontmatter.date);

    Some(Post {
        slug,
        title: frontmatter.title,
        date: frontmatter.date,
        date_display,
        date_short,
        excerpt: frontmatter.excerpt,
        content,
        tags: frontmatter.tags,
        read_time: frontmatter.read_time,
        image: frontmatter.image,
    })
}

/// Load all posts from the `content/` directory, sorted newest-first.
fn load_all_posts() -> Vec<Post> {
    let content_dir = Path::new("content");
    let mut posts: Vec<Post> = fs::read_dir(content_dir)
        .unwrap_or_else(|_| panic!("content/ directory not found"))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()?.to_str()? == "md" {
                load_post(&path)
            } else {
                None
            }
        })
        .collect();

    // ISO date strings sort lexicographically — newest first
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    featured: Post,
    posts: Vec<Post>,
    page: i32,
    total_pages: i32,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: Post,
}

// ── Axum / Askama glue ────────────────────────────────────────────────────────

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {}", err),
            )
                .into_response(),
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

const POSTS_PER_PAGE: usize = 6;

#[derive(Deserialize)]
struct Pagination {
    page: Option<i32>,
}

async fn home(Query(params): Query<Pagination>) -> impl IntoResponse {
    let all_posts = load_all_posts();

    if all_posts.is_empty() {
        return (StatusCode::OK, Html("<p>No posts yet.</p>".to_string())).into_response();
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

async fn post_page(AxumPath(slug): AxumPath<String>) -> impl IntoResponse {
    let content_dir = Path::new("content");
    let found = fs::read_dir(content_dir)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .find(|p| {
                    p.extension().and_then(|e| e.to_str()) == Some("md")
                        && p.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s == slug || s.ends_with(&format!("-{}", slug)))
                            .unwrap_or(false)
                })
        });

    match found.and_then(|p| load_post(&p)) {
        Some(post) => HtmlTemplate(PostTemplate { post }).into_response(),
        None => (StatusCode::NOT_FOUND, "Post not found").into_response(),
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/posts/{slug}", get(post_page))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/public", ServeDir::new("public"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
