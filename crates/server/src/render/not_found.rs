//! The page for addresses that do not exist. It has no canonical address and no JSON-LD.

use cuhlumn_api::site::theme::HomeLayout;
use maud::html;

use crate::render::{
    blog::{Blog, Entry},
    layout::page,
    listing,
    meta::Head,
};

const LATEST: usize = 5;

/// Returns the "Page not found" page with the latest posts.
#[must_use]
pub fn render(blog: &Blog) -> String {
    let site = &blog.site;
    let latest: Vec<&Entry> = blog.articles().take(LATEST).collect();
    let head = Head {
        index: false,
        ..Head::new(
            format!("Page not found · {}", site.title),
            "Page not found",
            site.description.clone(),
            blog.site_image(),
        )
    };
    let body = html! {
        h1 { "Page not found" }
        p.intro { "No page lives at this address. Try one of the latest posts." }
        (listing::posts(&latest, false, HomeLayout::List))
    };
    page(blog, &head, body)
}
