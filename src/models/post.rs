use serde::Deserialize;

/// Matches the YAML frontmatter fields in each .md file under content/
#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: String,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub read_time: i32,
    pub image: String,
}

#[derive(Debug, Clone)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub date: String,         // "2025-05-01" — used for sorting
    pub date_display: String, // "May 1, 2025" — rendered in templates
    pub date_short: String,   // "May 1"       — rendered on cards
    pub excerpt: String,
    pub content: String,
    pub tags: Vec<String>,
    pub read_time: i32,
    pub image: String,
}

impl Post {
    pub fn primary_tag(&self) -> &str {
        self.tags.first().map(|s| s.as_str()).unwrap_or("general")
    }
}
