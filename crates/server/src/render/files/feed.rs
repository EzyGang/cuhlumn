//! The Atom feed of the latest posts.

use std::fmt;

use crate::render::{
    blog::{Blog, Entry},
    escape, iso,
};

/// The feed lists at most this number of posts.
const FEED_POSTS: usize = 50;

/// Returns the Atom feed of the latest 50 posts, without pages.
#[must_use]
pub fn render(blog: &Blog) -> String {
    Feed(blog).to_string()
}

struct Feed<'a>(&'a Blog);

impl fmt::Display for Feed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blog = self.0;
        let site = &blog.site;
        let author = match site.author.trim() {
            "" => site.title.as_str(),
            author => author,
        };
        writeln!(f, "<?xml version=\"1.0\" encoding=\"utf-8\"?>")?;
        writeln!(
            f,
            "<feed xmlns=\"http://www.w3.org/2005/Atom\" xml:lang=\"{}\">",
            escape(site.lang.as_str())
        )?;
        writeln!(f, "<title>{}</title>", escape(&site.title))?;
        writeln!(f, "<subtitle>{}</subtitle>", escape(&site.description))?;
        writeln!(f, "<link href=\"{}/\"/>", blog.origin)?;
        writeln!(f, "<link rel=\"self\" href=\"{}/feed.xml\"/>", blog.origin)?;
        writeln!(f, "<id>{}/</id>", blog.origin)?;
        writeln!(f, "<updated>{}</updated>", iso(blog.updated_at()))?;
        writeln!(f, "<author><name>{}</name></author>", escape(author))?;
        writeln!(f, "<generator>Cuhlumn</generator>")?;
        for entry in blog.articles().take(FEED_POSTS) {
            write_entry(f, blog, entry)?;
        }
        writeln!(f, "</feed>")
    }
}

fn write_entry(f: &mut fmt::Formatter<'_>, blog: &Blog, entry: &Entry) -> fmt::Result {
    let post = &entry.post;
    let content = absolutize(&entry.doc().html, &blog.origin);
    writeln!(f, "<entry>")?;
    writeln!(f, "<title>{}</title>", escape(&post.title))?;
    writeln!(f, "<link href=\"{}\"/>", post.url)?;
    writeln!(f, "<id>{}</id>", post.url)?;
    let published = post.published_at.unwrap_or(post.updated_at);
    writeln!(f, "<published>{}</published>", iso(published))?;
    writeln!(f, "<updated>{}</updated>", iso(post.updated_at))?;
    writeln!(f, "<summary>{}</summary>", escape(&entry.summary()))?;
    writeln!(f, "<content type=\"html\">{}</content>", escape(&content))?;
    writeln!(f, "</entry>")
}

/// Prefixes root-relative `src` and `href` values with `origin`, so feed readers can load them.
///
/// Protocol-relative values such as `//cdn.example.com/x.png` stay as they are.
fn absolutize(html: &str, origin: &str) -> String {
    const ATTRIBUTES: [&str; 2] = ["src=\"/", "href=\"/"];
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        let next = ATTRIBUTES
            .iter()
            .filter_map(|attribute| rest.find(attribute).map(|at| at + attribute.len() - 1))
            .min();
        let Some(slash) = next else {
            break;
        };
        let (head, tail) = rest.split_at(slash);
        out.push_str(head);
        if !tail.starts_with("//") {
            out.push_str(origin);
        }
        out.push('/');
        rest = &tail[1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use crate::render::files::feed::absolutize;

    #[test]
    fn root_relative_addresses_become_absolute() {
        let html = r##"<a href="/about/">a</a><img src="/media/x.png"><a href="//cdn.example/y">b</a><a href="#top">c</a><a href="https://x.org/">d</a>"##;
        assert_eq!(
            absolutize(html, "https://blog.example.com"),
            r##"<a href="https://blog.example.com/about/">a</a><img src="https://blog.example.com/media/x.png"><a href="//cdn.example/y">b</a><a href="#top">c</a><a href="https://x.org/">d</a>"##
        );
    }
}
