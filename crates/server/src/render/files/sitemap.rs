//! The XML sitemap.

use std::fmt;

use crate::render::{blog::Blog, iso};

/// Returns the sitemap: the home page and every published entry except the page shown as the home page.
#[must_use]
pub fn render(blog: &Blog) -> String {
    Sitemap(blog).to_string()
}

struct Sitemap<'a>(&'a Blog);

impl fmt::Display for Sitemap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blog = self.0;
        writeln!(f, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(
            f,
            "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"
        )?;
        write_url(f, &blog.root_url(), blog.updated_at())?;
        let entries = blog
            .entries()
            .iter()
            .filter(|entry| !blog.is_home(&entry.post.slug));
        for entry in entries {
            write_url(f, &entry.post.url, entry.post.updated_at)?;
        }
        writeln!(f, "</urlset>")
    }
}

fn write_url(f: &mut fmt::Formatter<'_>, url: &str, modified: i64) -> fmt::Result {
    writeln!(
        f,
        "<url><loc>{url}</loc><lastmod>{}</lastmod></url>",
        iso(modified)
    )
}
