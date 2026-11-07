//! The page of posts that carry one tag. Search engines do not index it.

use maud::html;

use crate::render::{
    blog::{Blog, Entry},
    layout::{page, trail},
    listing,
    markdown::slugify,
    meta::Head,
    schema,
};

/// Returns the posts whose tag has the address slug `slug`, or `None` when no post has it.
#[must_use]
pub fn render(blog: &Blog, slug: &str) -> Option<String> {
    let site = &blog.site;
    let posts: Vec<&Entry> = blog
        .articles()
        .filter(|entry| tag_name(entry, slug).is_some())
        .collect();
    let name = tag_name(posts.first()?, slug)?;
    let url = format!("{}/tags/{slug}/", blog.origin);
    let head = Head {
        index: false,
        schema: Some(schema::tag(name, &url)),
        url: Some(url),
        ..Head::new(
            format!("#{name} · {}", site.title),
            name,
            format!("Posts about {name} on {}.", site.title),
            blog.site_image(),
        )
    };
    let body = html! {
        (trail(blog))
        h1 { "#" (name) }
        (listing::posts(&posts, false, site.brand.home_layout))
    };
    Some(page(blog, &head, body))
}

/// Returns `true` when a published post has a tag with the address slug `slug`.
#[must_use]
pub fn exists(blog: &Blog, slug: &str) -> bool {
    blog.articles().any(|entry| tag_name(entry, slug).is_some())
}

/// Returns the spelling of the entry tag whose address slug is `slug`.
fn tag_name<'a>(entry: &'a Entry, slug: &str) -> Option<&'a str> {
    let tags = entry.post.tags.as_slice();
    tags.iter()
        .map(|tag| tag.as_str())
        .find(|tag| slugify(tag) == slug)
}
