//! The frame of every page: head, site header, footer, and the post list.

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::render::{
    blog::{Blog, Entry},
    day, initial, iso,
    meta::Head,
    schema,
    style::{DARK, LIGHT, STATIC_CSS, site_css},
};

const THEME_INIT_JS: &str = include_str!("../../assets/theme-init.js");
const THEME_TOGGLE_JS: &str = include_str!("../../assets/theme-toggle.js");
const CONTRAST_SVG: &str = include_str!("../../assets/contrast.svg");

/// Returns a full HTML document with `body` inside the main element.
#[must_use]
pub fn page(blog: &Blog, head: &Head<'_>, body: Markup) -> String {
    let site = &blog.site;
    let brand = &site.brand;
    let icon = brand.favicon.as_ref().or(brand.logo.as_ref());
    html! {
        (DOCTYPE)
        html lang=(site.lang) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                (head.document_tags())
                meta name="color-scheme" content="light dark";
                meta name="theme-color" media="(prefers-color-scheme: light)" content=(LIGHT.bg.hex());
                meta name="theme-color" media="(prefers-color-scheme: dark)" content=(DARK.bg.hex());
                @if let Some(icon) = icon { link rel="icon" href=(icon); }
                @else { link rel="icon" href="/favicon.svg" type="image/svg+xml"; }
                link rel="alternate" type="application/atom+xml" title=(site.title) href="/feed.xml";
                @if let Some(markdown) = &head.markdown { link rel="alternate" type="text/markdown" href=(markdown); }
                (head.social_tags(&site.title))
                @if let Some(data) = &head.schema { (schema::script(data)) }
                script { (PreEscaped(THEME_INIT_JS)) }
                style { (PreEscaped(STATIC_CSS)) (PreEscaped(site_css(&brand.accent, brand.font))) }
                (PreEscaped(&brand.head))
            }
            body {
                a.skip-link href="#content" { "Skip to content" }
                (site_head(blog))
                main #content .column { (body) }
                (site_foot(blog))
                script { (PreEscaped(THEME_TOGGLE_JS)) }
            }
        }
    }
    .into_string()
}

/// Returns a list of posts with dates and summaries. Nested lists use `h3` titles.
#[must_use]
pub fn post_list(entries: &[&Entry], nested: bool) -> Markup {
    if entries.is_empty() {
        return html! { p.entry-summary { "No posts yet." } };
    }
    html! {
        ol.entry-list { @for entry in entries {
            @let post = &entry.post;
            @let link = html! { a href={ "/" (post.slug) "/" } { (post.title) } };
            li {
                @if let Some(time) = post.published_at {
                    time.entry-date datetime=(iso(time)) { (day(time)) }
                }
                @if nested { h3.entry-title { (link) } } @else { h2.entry-title { (link) } }
                p.entry-summary { (entry.summary()) }
            }
        } }
    }
}

/// Returns the breadcrumb link back to the home page.
#[must_use]
pub fn trail(blog: &Blog) -> Markup {
    html! { nav.trail aria-label="Breadcrumb" { a href="/" { (blog.site.title) } } }
}

fn site_head(blog: &Blog) -> Markup {
    let site = &blog.site;
    html! {
        header.site-head { div.column.site-head-row {
            a.site-brand href="/" {
                @if let Some(logo) = &site.brand.logo { img.brand-logo src=(logo) alt="" width="28" height="28"; }
                @else { span.brand-badge aria-hidden="true" { (initial(&site.title)) } }
                span { (site.title) }
            }
            div.site-tools {
                nav.site-nav aria-label="Main" {
                    @for page in blog.pages() { a href={ "/" (page.post.slug) "/" } { (page.post.title) } }
                    @for link in &site.brand.links { a href=(link.url) { (link.label) } }
                    @if let Some((label, url)) = blog.project() { a.nav-project href=(url) { (label) } }
                }
                button #theme-switch .theme-switch type="button" aria-label="Dark theme" aria-pressed="false" {
                    (PreEscaped(CONTRAST_SVG))
                }
            }
        } }
    }
}

fn site_foot(blog: &Blog) -> Markup {
    let site = &blog.site;
    html! {
        footer.site-foot { div.column {
            p {
                strong { (site.title) }
                @if !site.description.is_empty() { " — " (site.description) }
            }
            nav.foot-links aria-label="Feeds and indexes" {
                a href="/feed.xml" { "Atom feed" }
                a href="/sitemap.xml" { "Sitemap" }
                a href="/llms.txt" { "llms.txt" }
            }
            @if !site.brand.footer.is_empty() { p { (site.brand.footer) } }
            @if site.brand.credit { p { "Published with Cuhlumn" } }
        } }
    }
}
