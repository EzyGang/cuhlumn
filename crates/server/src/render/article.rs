//! The page of one post or page.

use maud::{Markup, PreEscaped, html};

use crate::render::{
    blog::{Blog, Entry},
    day, iso,
    layout::{page, post_list, trail},
    markdown::slugify,
    meta::{Head, OgType},
    schema,
};

/// A post edited more than this number of seconds after publication shows its update day.
const EDIT_NOTICE_SECONDS: i64 = 86_400;
const MORE_POSTS: usize = 3;

/// Returns the page of a published post or page.
#[must_use]
pub fn render(blog: &Blog, entry: &Entry) -> String {
    let (site, post) = (&blog.site, &entry.post);
    let doc = entry.doc();
    let description = entry.summary();
    let image = blog.entry_image(entry);
    let is_page = entry.is_page();
    let head = Head {
        url: Some(post.url.clone()),
        kind: if is_page {
            OgType::Website
        } else {
            OgType::Article
        },
        published: post.published_at.filter(|_| !is_page),
        modified: Some(post.updated_at),
        tags: post.tags.as_slice(),
        markdown: Some(blog.markdown_url(entry)),
        schema: Some(schema::article(blog, entry, &description, &image.url)),
        ..Head::new(
            format!("{} · {}", post.title, site.title),
            &post.title,
            description,
            image,
        )
    };
    let body = html! {
        @if !is_page { div.read-progress aria-hidden="true" {} }
        (trail(blog))
        article {
            header.post-head {
                h1 { (post.title) }
                @if !post.description.trim().is_empty() { p.standfirst { (post.description) } }
                @if !is_page { (byline(blog, entry)) }
            }
            // Covers are share-sized (1200×630). The width and height reserve the space before the image loads.
            @if let Some(cover) = &post.cover {
                img.hero-image src=(cover) alt=(post.title) width="1200" height="630" fetchpriority="high";
            }
            @if doc.shows_toc() {
                details.contents open {
                    summary { "Contents" }
                    ol { @for item in &doc.toc {
                        li class={ "depth-" (item.level) } { a href={ "#" (item.id) } { (item.label) } }
                    } }
                }
            }
            div.post-body { (PreEscaped(&doc.html)) }
            @if !post.tags.as_slice().is_empty() {
                ul.tag-chips aria-label="Tags" { @for tag in post.tags.as_slice() {
                    li { a href={ "/tags/" (slugify(tag.as_str())) "/" } { "#" (tag) } }
                } }
            }
        }
        @if let Some((label, url)) = blog.project() {
            aside.pitch {
                @if !site.cta.is_empty() { p { (site.cta) } }
                a.pitch-button href=(url) { (label) }
            }
        }
        @if !is_page { (more_posts(blog, entry)) }
    };
    page(blog, &head, body)
}

/// Returns the day, the update day, the reading time, and the author of a post.
fn byline(blog: &Blog, entry: &Entry) -> Markup {
    let (site, post) = (&blog.site, &entry.post);
    let edited = post
        .published_at
        .is_some_and(|time| post.updated_at.saturating_sub(time) > EDIT_NOTICE_SECONDS);
    html! {
        p.byline {
            @if let Some(time) = post.published_at { time datetime=(iso(time)) { (day(time)) } }
            @if edited { span { "updated " time datetime=(iso(post.updated_at)) { (day(post.updated_at)) } } }
            span { (entry.minutes()) " min read" }
            @if !site.author.is_empty() {
                @if let Some(url) = &site.author_url { a href=(url) rel="author" { (site.author) } }
                @else { span { (site.author) } }
            }
        }
    }
}

/// Returns up to three other posts: posts with a shared tag first, otherwise the latest posts.
fn more_posts(blog: &Blog, entry: &Entry) -> Markup {
    let post = &entry.post;
    let others = || blog.articles().filter(|other| other.post.slug != post.slug);
    let mut related: Vec<&Entry> = others()
        .filter(|other| {
            let tags = other.post.tags.as_slice();
            tags.iter().any(|tag| post.tags.as_slice().contains(tag))
        })
        .take(MORE_POSTS)
        .collect();
    if related.is_empty() {
        related = others().take(MORE_POSTS).collect();
    }
    if related.is_empty() {
        return html! {};
    }
    html! { section { h2.list-heading { "More posts" } (post_list(&related, true)) } }
}
