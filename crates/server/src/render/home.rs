//! The home page: the post list, or the home page of the site with the latest posts below it.

use maud::{PreEscaped, html};

use crate::render::{
    blog::{Blog, Entry},
    layout::page,
    listing,
    meta::Head,
    schema,
};

/// Returns the home page.
#[must_use]
pub fn render(blog: &Blog) -> String {
    let site = &blog.site;
    let posts: Vec<&Entry> = blog.articles().collect();
    let home = blog.home_page();
    let description = home.map_or_else(|| site.description.clone(), Entry::summary);
    let head = Head {
        url: Some(blog.root_url()),
        schema: Some(schema::home(blog, &description)),
        ..Head::new(
            home.map_or_else(|| site.title.clone(), |entry| entry.post.title.clone()),
            home.map_or(&site.title, |entry| &entry.post.title),
            description,
            blog.site_image(),
        )
    };
    let body = match home {
        Some(entry) => html! {
            h1 { (entry.post.title) }
            div.post-body { (PreEscaped(&entry.doc().html)) }
            @if !posts.is_empty() { h2.list-heading { "Latest posts" } (listing::posts(&posts, true, site.brand.home_layout)) }
        },
        None => html! {
            h1 { (site.title) }
            @if !site.description.is_empty() { p.intro { (site.description) } }
            (listing::posts(&posts, false, site.brand.home_layout))
        },
    };
    page(blog, &head, body)
}
