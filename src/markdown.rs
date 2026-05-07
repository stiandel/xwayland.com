use std::{fs, path::Path};

use gray_matter::{engine::YAML, Matter, ParsedEntity, Pod};
use pulldown_cmark::{html, Options, Parser};

use crate::models::post::{Frontmatter, Post};

/// Parse a YYYY-MM-DD date string into ("May 1, 2025", "May 1").
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
pub fn markdown_to_html(md: &str) -> String {
    let opts = Options::all();
    let parser = Parser::new_ext(md, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

/// Load and parse a single .md file. Returns None if missing or malformed.
pub fn load_post(path: &Path) -> Option<Post> {
    let raw = fs::read_to_string(path).ok()?;

    let matter = Matter::<YAML>::new();
    let parsed: ParsedEntity<Pod> = matter.parse(&raw).ok()?;

    let frontmatter: Frontmatter = parsed.data?.deserialize().ok()?;
    let content = markdown_to_html(&parsed.content);

    // Derive slug from filename: "2025-05-01-my-post.md" → "my-post"
    let filename = path.file_stem()?.to_string_lossy();
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

/// Load all posts from `content/`, sorted newest-first.
pub fn load_all_posts() -> Vec<Post> {
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

    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

/// Find a single post by slug, scanning content/.
pub fn find_post(slug: &str) -> Option<Post> {
    let content_dir = Path::new("content");
    let path = fs::read_dir(content_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("md")
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s == slug || s.ends_with(&format!("-{}", slug)))
                    .unwrap_or(false)
        })?;

    load_post(&path)
}
