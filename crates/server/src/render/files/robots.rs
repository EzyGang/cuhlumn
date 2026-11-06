//! The `robots.txt` file.

use crate::render::blog::Blog;

/// Returns `robots.txt`: everything except the API may be crawled, and the sitemap address.
#[must_use]
pub fn render(blog: &Blog) -> String {
    format!(
        "User-agent: *\nAllow: /\nDisallow: /api/\n\nSitemap: {}/sitemap.xml\n",
        blog.origin
    )
}
