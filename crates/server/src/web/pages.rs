//! The response for each public route of a blog.

use std::sync::Arc;

use crate::{
    og::{self, CardText, OgError},
    render::{
        article,
        blog::{Blog, Entry},
        files::{copy, favicon, feed, llms, robots, sitemap},
        home, tag,
    },
    state::AppState,
    web::{
        cache::Cached,
        route::{Route, encode_segment},
    },
};

/// What a route answers.
pub enum Outcome {
    /// A response to cache until the next write to the site.
    Page(Cached),
    /// A permanent redirect to a percent-encoded location.
    Redirect(String),
    /// The HTML "Page not found" page.
    Missing,
}

/// A failure while a page builds.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// A share card cannot be drawn.
    #[error(transparent)]
    Card(#[from] OgError),
    /// The blocking build task did not finish.
    #[error("the page build task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

const HTML: &str = "text/html; charset=utf-8";
const TEXT: &str = "text/plain; charset=utf-8";
const MARKDOWN: &str = "text/markdown; charset=utf-8";
const XML: &str = "application/xml; charset=utf-8";
const ATOM: &str = "application/atom+xml; charset=utf-8";
const SVG: &str = "image/svg+xml";
const PNG: &str = "image/png";
const PAGE_MAX_AGE: u32 = 60;
const FILE_MAX_AGE: u32 = 300;
const IMAGE_MAX_AGE: u32 = 86_400;

/// Builds the answer of `route` on `blog`. Rendering, hashing, and compression run on the blocking thread pool.
///
/// `query` is the raw query string of the request. Slash redirects keep it.
///
/// # Errors
///
/// Returns [`BuildError`] when a share card cannot be drawn or a blocking task fails.
pub async fn build(
    state: &AppState,
    blog: Arc<Blog>,
    route: Route,
    query: Option<String>,
) -> Result<Outcome, BuildError> {
    let text = match &route {
        Route::SiteCard => CardText {
            title: blog.site.title.clone(),
            subtitle: blog.site.description.clone(),
            meta: String::new(),
        },
        Route::EntryCard(slug) => match blog.find(slug) {
            Some(entry) => entry_card(entry),
            None => return Ok(Outcome::Missing),
        },
        _ => {
            let task = move || render(&blog, route, query.as_deref());
            return Ok(tokio::task::spawn_blocking(task).await?);
        }
    };
    let fonts = state.config().fonts.clone();
    let card = og::prepare(state.media(), fonts, &blog.site, text).await?;
    let task = move || {
        card.draw()
            .map(|png| Outcome::Page(Cached::new(PNG, png, IMAGE_MAX_AGE)))
    };
    Ok(tokio::task::spawn_blocking(task).await??)
}

/// Builds the answer of every route that needs no share card.
///
/// Share cards are drawn by [`build`], and the caller streams media files, so those routes are missing here.
fn render(blog: &Blog, route: Route, query: Option<&str>) -> Outcome {
    let redirect = |path: String| match query {
        Some(query) => Outcome::Redirect(format!("{path}?{query}")),
        None => Outcome::Redirect(path),
    };
    match route {
        Route::Home => html(home::render(blog)),
        Route::Entry(slug) | Route::Bare(slug) if blog.is_home(&slug) => redirect("/".to_owned()),
        Route::Entry(slug) => blog
            .find(&slug)
            .map_or(Outcome::Missing, |entry| html(article::render(blog, entry))),
        Route::Bare(slug) => match blog.find(&slug) {
            Some(_) => redirect(format!("/{}/", encode_segment(slug.as_str()))),
            None => Outcome::Missing,
        },
        Route::Markdown(slug) => blog.find(&slug).map_or(Outcome::Missing, |entry| {
            file(MARKDOWN, copy::render(blog, entry))
        }),
        Route::Tag(slug) => tag::render(blog, &slug).map_or(Outcome::Missing, html),
        Route::BareTag(slug) if tag::exists(blog, &slug) => {
            redirect(format!("/tags/{}/", encode_segment(&slug)))
        }
        Route::Feed => file(ATOM, feed::render(blog)),
        Route::Sitemap => file(XML, sitemap::render(blog)),
        Route::Robots => file(TEXT, robots::render(blog)),
        Route::Llms => file(TEXT, llms::render(blog)),
        Route::LlmsFull => file(TEXT, llms::render_full(blog)),
        Route::Favicon => {
            Outcome::Page(Cached::new(SVG, favicon::render(&blog.site), IMAGE_MAX_AGE))
        }
        Route::FaviconIco => Outcome::Redirect("/favicon.svg".to_owned()),
        Route::KeyFile(key) if key == blog.site.indexnow_key => file(TEXT, key.into()),
        Route::BareTag(_)
        | Route::KeyFile(_)
        | Route::SiteCard
        | Route::EntryCard(_)
        | Route::Media(_)
        | Route::NotFound => Outcome::Missing,
    }
}

fn html(body: String) -> Outcome {
    Outcome::Page(Cached::new(HTML, body, PAGE_MAX_AGE))
}

fn file(content_type: &'static str, body: String) -> Outcome {
    Outcome::Page(Cached::new(content_type, body, FILE_MAX_AGE))
}

fn entry_card(entry: &Entry) -> CardText {
    CardText {
        title: entry.post.title.clone(),
        subtitle: entry.post.description.clone(),
        meta: entry.card_meta(),
    }
}
