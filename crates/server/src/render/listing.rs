//! Lists of posts in the three home layouts: list, compact, and cards.

use cuhlumn_api::site::theme::HomeLayout;
use maud::{Markup, html};

use crate::render::{blog::Entry, day, iso};

/// Cards after this many load their cover only near the viewport.
const EAGER_COVERS: usize = 2;

/// Returns a list of posts in `layout`. Nested lists use `h3` titles.
#[must_use]
pub fn posts(entries: &[&Entry], nested: bool, layout: HomeLayout) -> Markup {
    if entries.is_empty() {
        return html! { p.entry-summary { "No posts yet." } };
    }
    match layout {
        HomeLayout::List => list(entries, nested),
        HomeLayout::Compact => compact(entries, nested),
        HomeLayout::Cards => cards(entries, nested),
    }
}

/// Dates, titles, and summaries, one below the other.
fn list(entries: &[&Entry], nested: bool) -> Markup {
    html! {
        ol.entry-list { @for entry in entries {
            li { (date(entry)) (title(entry, nested)) p.entry-summary { (entry.summary()) } }
        } }
    }
}

/// One line for each post: the date and the title.
fn compact(entries: &[&Entry], nested: bool) -> Markup {
    html! {
        ol.entry-list.entry-compact { @for entry in entries {
            li { (date(entry)) (title(entry, nested)) }
        } }
    }
}

/// A grid of cards with the cover image when the post has one.
///
/// The first cover is often the largest image of the page, so it loads first.
fn cards(entries: &[&Entry], nested: bool) -> Markup {
    html! {
        ol.entry-cards { @for (index, entry) in entries.iter().enumerate() {
            li.entry-card {
                @if let Some(cover) = &entry.post.cover {
                    @let lazy = (index >= EAGER_COVERS).then_some("lazy");
                    @let priority = (index == 0).then_some("high");
                    img.entry-cover src=(cover) alt="" width="1200" height="630" loading=[lazy] fetchpriority=[priority];
                }
                div.entry-card-body { (date(entry)) (title(entry, nested)) p.entry-summary { (entry.summary()) } }
            }
        } }
    }
}

fn date(entry: &Entry) -> Markup {
    html! {
        @if let Some(time) = entry.post.published_at { time.entry-date datetime=(iso(time)) { (day(time)) } }
    }
}

fn title(entry: &Entry, nested: bool) -> Markup {
    let post = &entry.post;
    let link = html! { a href={ "/" (post.slug) "/" } { (post.title) } };
    html! { @if nested { h3.entry-title { (link) } } @else { h2.entry-title { (link) } } }
}
