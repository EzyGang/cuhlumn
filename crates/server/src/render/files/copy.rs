//! The Markdown copy of an entry, served next to its page at `/{slug}.md`.

use std::fmt;

use crate::render::{
    blog::{Blog, Entry},
    day,
};

/// Returns the entry as Markdown with its title, description, and address.
#[must_use]
pub fn render(blog: &Blog, entry: &Entry) -> String {
    MarkdownCopy { blog, entry }.to_string()
}

struct MarkdownCopy<'a> {
    blog: &'a Blog,
    entry: &'a Entry,
}

impl fmt::Display for MarkdownCopy<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (site, post) = (&self.blog.site, &self.entry.post);
        write!(f, "# {}\n\n", post.title)?;
        if !post.description.is_empty() {
            write!(f, "> {}\n\n", post.description)?;
        }
        match post.published_at {
            Some(time) if !self.entry.is_page() => write!(
                f,
                "Published {} on {}: {}\n\n",
                day(time),
                site.title,
                post.url
            )?,
            _ => write!(f, "From {}: {}\n\n", site.title, post.url)?,
        }
        write!(f, "{}\n\n", post.body.trim_end())
    }
}
