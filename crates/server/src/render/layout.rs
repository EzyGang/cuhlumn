//! The frame of every page: head, announcement bar, site header, and footer.

use cuhlumn_api::site::{brand::Brand, theme::Theme};
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{
    db::unix_now,
    render::{
        announcement::Bar,
        blog::Blog,
        initial,
        meta::Head,
        schema, social,
        style::{css::stylesheet, palettes::variants},
    },
};

const THEME_INIT_JS: &str = include_str!("../../assets/theme-init.js");
const THEME_TOGGLE_JS: &str = include_str!("../../assets/theme-toggle.js");
const CONTRAST_SVG: &str = include_str!("../../assets/contrast.svg");

/// Returns a full HTML document with `body` inside the main element.
///
/// A forced theme is the `data-theme` attribute of the root element, so the first paint already has it.
/// With the toggle, a stored choice replaces it before the first paint.
#[must_use]
pub fn page(blog: &Blog, head: &Head<'_>, body: Markup) -> String {
    let site = &blog.site;
    let brand = &site.brand;
    let icon = brand.favicon.as_ref().or(brand.logo.as_ref());
    let bar = Bar::live(site.announcement.as_ref(), unix_now());
    html! {
        (DOCTYPE)
        html lang=(site.lang) data-theme=[forced(brand.theme)] {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                (head.document_tags())
                (theme_tags(brand))
                @if let Some(icon) = icon { link rel="icon" href=(icon); }
                @else { link rel="icon" href="/favicon.svg" type="image/svg+xml"; }
                link rel="alternate" type="application/atom+xml" title=(site.title) href="/feed.xml";
                @if let Some(markdown) = &head.markdown { link rel="alternate" type="text/markdown" href=(markdown); }
                (head.social_tags(&site.title))
                @if let Some(data) = &head.schema { (schema::script(data)) }
                @if brand.theme_toggle { script { (PreEscaped(THEME_INIT_JS)) } }
                @if let Some(bar) = &bar { (bar.head()) }
                style { (PreEscaped(stylesheet(brand))) }
                (PreEscaped(&brand.head))
            }
            body {
                @if let Some(bar) = &bar { (bar.body()) }
                a.skip-link href="#content" { "Skip to content" }
                (site_head(blog))
                main #content .column { (body) }
                (site_foot(blog))
                @if brand.theme_toggle { script { (PreEscaped(THEME_TOGGLE_JS)) } }
                @if let Some(bar) = &bar { (bar.script()) }
            }
        }
    }
    .into_string()
}

/// Returns the breadcrumb link back to the home page.
#[must_use]
pub fn trail(blog: &Blog) -> Markup {
    html! { nav.trail aria-label="Breadcrumb" { a href="/" { (blog.site.title) } } }
}

/// Returns the theme that the site forces, or `None` when the system chooses.
const fn forced(theme: Theme) -> Option<&'static str> {
    match theme {
        Theme::System => None,
        Theme::Light => Some("light"),
        Theme::Dark => Some("dark"),
    }
}

/// Returns the `color-scheme` and `theme-color` tags of the theme option.
fn theme_tags(brand: &Brand) -> Markup {
    let pair = variants(brand.palette);
    let (light, dark) = (pair.light.bg.hex(), pair.dark.bg.hex());
    match brand.theme {
        Theme::System => html! {
            meta name="color-scheme" content="light dark";
            meta name="theme-color" media="(prefers-color-scheme: light)" content=(light);
            meta name="theme-color" media="(prefers-color-scheme: dark)" content=(dark);
        },
        Theme::Light => html! {
            meta name="color-scheme" content="light";
            meta name="theme-color" content=(light);
        },
        Theme::Dark => html! {
            meta name="color-scheme" content="dark";
            meta name="theme-color" content=(dark);
        },
    }
}

fn site_head(blog: &Blog) -> Markup {
    let site = &blog.site;
    let brand = &site.brand;
    html! {
        header.site-head { div.column.site-head-row {
            a.site-brand href="/" {
                @if let Some(logo) = &brand.logo { img.brand-logo src=(logo) alt="" width="28" height="28"; }
                @else { span.brand-badge aria-hidden="true" { (initial(&site.title)) } }
                span { (site.title) }
            }
            div.site-tools {
                nav.site-nav aria-label="Main" {
                    @for page in blog.pages() { a href={ "/" (page.post.slug) "/" } { (page.post.title) } }
                    @for link in &brand.links { a href=(link.url) { (link.label) } }
                    @if let Some((label, url)) = blog.project() { a.nav-project href=(url) { (label) } }
                }
                @if brand.social_in_header { (social::links(&brand.social)) }
                @if brand.theme_toggle {
                    button #theme-switch .theme-switch type="button" aria-label="Dark theme" aria-pressed="false" {
                        (PreEscaped(CONTRAST_SVG))
                    }
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
            (social::links(&site.brand.social))
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
