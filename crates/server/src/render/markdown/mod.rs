//! Markdown to HTML with heading anchors, a table of contents, lazy images, and tables that read on a phone.

mod headings;
mod tables;

use cuhlumn_api::wire::TocEntry;
use maud::html;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd};

use crate::render::markdown::headings::{Headings, OpenHeading};

/// Rendered Markdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doc {
    pub html: String,
    /// The H2 and H3 headings in document order.
    pub toc: Vec<TocEntry>,
    /// The plain text, with one space between words.
    pub text: String,
    pub words: usize,
}

/// The table of contents shows only when it has at least this number of entries.
pub const TOC_MIN_ENTRIES: usize = 3;

const OPTIONS: Options = Options::ENABLE_TABLES
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_SMART_PUNCTUATION)
    .union(Options::ENABLE_HEADING_ATTRIBUTES);

/// Task list markers with an accessible name, because the item text is not a `<label>`.
const TASK_DONE: &str = r#"<input type="checkbox" disabled checked aria-label="Completed task"> "#;
const TASK_OPEN: &str = r#"<input type="checkbox" disabled aria-label="Open task"> "#;

/// Renders Markdown. Raw HTML passes through, because only the token holder writes posts.
#[must_use]
pub fn render(markdown: &str) -> Doc {
    let mut builder = Builder::default();
    for event in Parser::new_ext(markdown, OPTIONS) {
        builder.push(event);
    }
    builder.finish()
}

/// Returns lowercase words joined by hyphens, for heading ids and tag addresses.
///
/// Text without letters or digits gives `section`.
#[must_use]
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    for character in text.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            slug.push(character);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        return "section".to_owned();
    }
    slug
}

#[derive(Default)]
struct Builder<'a> {
    events: Vec<Event<'a>>,
    text: String,
    headings: Headings,
    heading: Option<OpenHeading<'a>>,
    image: Option<OpenImage<'a>>,
}

/// An image whose alternative text is still being read.
struct OpenImage<'a> {
    src: CowStr<'a>,
    title: CowStr<'a>,
    alt: String,
    depth: usize,
}

impl Doc {
    /// Returns `true` when the page shows the table of contents.
    #[must_use]
    pub fn shows_toc(&self) -> bool {
        self.toc.len() >= TOC_MIN_ENTRIES
    }
}

impl<'a> Builder<'a> {
    fn push(&mut self, event: Event<'a>) {
        self.collect_text(&event);
        if let Some(image) = self.image.as_mut() {
            match &event {
                Event::Start(Tag::Image { .. }) => image.depth += 1,
                Event::End(TagEnd::Image) if image.depth > 0 => image.depth -= 1,
                Event::End(TagEnd::Image) => {
                    let html = image.html();
                    self.image = None;
                    self.emit(Event::InlineHtml(html.into()));
                }
                Event::Text(text) | Event::Code(text) => image.alt.push_str(text),
                _ => (),
            }
            return;
        }
        match event {
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                self.image = Some(OpenImage {
                    src: dest_url,
                    title,
                    alt: String::new(),
                    depth: 0,
                });
            }
            Event::Start(Tag::Heading {
                level,
                id,
                classes,
                attrs,
            }) => self.heading = Some(OpenHeading::new(level, id, classes, attrs)),
            Event::End(TagEnd::Heading(_)) => {
                if let Some(heading) = self.heading.take() {
                    self.headings.close(heading, &mut self.events);
                }
            }
            Event::TaskListMarker(done) => {
                let marker = if done { TASK_DONE } else { TASK_OPEN };
                self.emit(Event::InlineHtml(CowStr::Borrowed(marker)));
            }
            event => self.emit(event),
        }
    }

    fn emit(&mut self, event: Event<'a>) {
        match self.heading.as_mut() {
            Some(heading) => heading.push(event),
            None => self.events.push(event),
        }
    }

    /// Adds the text of an event to the plain text and to the label of an open heading.
    fn collect_text(&mut self, event: &Event<'a>) {
        match event {
            Event::Text(text) | Event::Code(text) => {
                self.text.push_str(text);
                if let Some(heading) = self.heading.as_mut() {
                    heading.push_label(text);
                }
            }
            Event::SoftBreak
            | Event::HardBreak
            | Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::TableCell
                | TagEnd::CodeBlock,
            ) => self.text.push(' '),
            _ => (),
        }
    }

    fn finish(self) -> Doc {
        let events = tables::transform(self.events);
        let mut html = String::with_capacity(self.text.len() * 2);
        pulldown_cmark::html::push_html(&mut html, events.into_iter());
        let words: Vec<&str> = self.text.split_whitespace().collect();
        Doc {
            html,
            toc: self.headings.into_toc(),
            words: words.len(),
            text: words.join(" "),
        }
    }
}

impl OpenImage<'_> {
    fn html(&self) -> String {
        let title = (!self.title.is_empty()).then_some(self.title.as_ref());
        html! {
            img loading="lazy" decoding="async" src=(self.src) alt=(self.alt) title=[title];
        }
        .into_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::render::markdown::render;

    #[test]
    fn images_load_lazily() {
        let doc = render("![A *red* door](/media/door.png \"Door\")");
        assert!(
            doc.html.contains(
                r#"<img loading="lazy" decoding="async" src="/media/door.png" alt="A red door" title="Door">"#
            ),
            "{}",
            doc.html
        );
    }

    #[test]
    fn headings_get_unique_ids_and_anchors() {
        let doc = render("## Why\n\n## Why\n\n### Deep {#own}\n\n#### Low\n\n## \"Quoted\"");
        assert!(
            doc.html
                .contains(r##"<h2 id="why">Why<a class="heading-link" href="#why""##)
        );
        assert!(doc.html.contains(r#"id="why-2""#));
        assert!(doc.html.contains(r#"<h3 id="own">"#));
        assert!(doc.html.contains(r#"<h4 id="low">Low</h4>"#));
        let ids: Vec<&str> = doc.toc.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(ids, ["why", "why-2", "own", "quoted"]);
        assert_eq!(doc.toc[2].level, 3);
        assert_eq!(doc.toc[3].label, "\u{201c}Quoted\u{201d}");
        assert!(doc.shows_toc());
    }

    #[test]
    fn text_counts_words() {
        let doc = render("One *two*\nthree.\n\n- four\n- `five`\n");
        assert_eq!(doc.text, "One two three. four five");
        assert_eq!(doc.words, 5);
        assert!(!doc.shows_toc());
    }

    #[test]
    fn task_list_checkboxes_have_accessible_names() {
        let doc = render("- [x] Move the API\n- [ ] Move the workers\n");
        assert!(
            doc.html.contains(
                r#"<input type="checkbox" disabled checked aria-label="Completed task"> Move the API"#
            ),
            "{}",
            doc.html
        );
        assert!(
            doc.html.contains(
                r#"<input type="checkbox" disabled aria-label="Open task"> Move the workers"#
            ),
            "{}",
            doc.html
        );
    }
}
