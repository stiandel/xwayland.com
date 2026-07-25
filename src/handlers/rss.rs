use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};

use crate::markdown::load_all_posts;

pub async fn rss_feed() -> Response {
    let posts = load_all_posts();

    let items: String = posts
        .iter()
        .map(|p| {
            let pub_date = rfc822_date(&p.date);
            let link = format!("https://xwayland.com/posts/{}", p.slug);
            format!(
                "    <item>\n      <title>{}</title>\n      <link>{}</link>\n      <guid>{}</guid>\n      <pubDate>{}</pubDate>\n      <description>{}</description>\n    </item>",
                escape_xml(&p.title),
                link,
                link,
                pub_date,
                escape_xml(&p.excerpt),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>xwayland.com</title>
    <link>https://xwayland.com</link>
    <description>Posts about Linux, Wayland, and software</description>
    <language>en-us</language>
{items}
  </channel>
</rss>"#
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

/// Convert "YYYY-MM-DD" to RFC 822 format required by RSS.
fn rfc822_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let month_idx: usize = parts[1].parse::<usize>().unwrap_or(1).saturating_sub(1);
    let month = months.get(month_idx).unwrap_or(&"Jan");
    format!("{} {} {} 00:00:00 +0000", parts[2], month, parts[0])
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
