//! The [llms.txt](https://llmstxt.org) files: what the site is, with links to the Markdown copies.

use std::fmt;

use crate::render::{blog::Blog, files::copy};

/// Returns `llms.txt`: the site, its project, and a list of posts and a list of pages.
#[must_use]
pub fn render(blog: &Blog) -> String {
    Llms(blog).to_string()
}

/// Returns `llms-full.txt`: `llms.txt` followed by the Markdown copy of every entry.
#[must_use]
pub fn render_full(blog: &Blog) -> String {
    let mut text = render(blog);
    for entry in blog.entries() {
        text.push_str("---\n\n");
        text.push_str(&copy::render(blog, entry));
    }
    text
}

struct Llms<'a>(&'a Blog);

impl fmt::Display for Llms<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blog = self.0;
        let site = &blog.site;
        write!(f, "# {}\n\n", site.title)?;
        if !site.description.is_empty() {
            write!(f, "> {}\n\n", site.description)?;
        }
        if let Some((label, url)) = blog.project() {
            write!(
                f,
                "This blog is published by the makers of [{label}]({url}).\n\n"
            )?;
        }
        for (heading, pages) in [("Posts", false), ("Pages", true)] {
            let mut items = blog
                .entries()
                .iter()
                .filter(|entry| entry.is_page() == pages)
                .peekable();
            if items.peek().is_none() {
                continue;
            }
            write!(f, "## {heading}\n\n")?;
            for entry in items {
                writeln!(
                    f,
                    "- [{}]({}): {}",
                    entry.post.title,
                    blog.markdown_url(entry),
                    entry.summary()
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
