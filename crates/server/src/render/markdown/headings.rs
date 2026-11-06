//! Heading ids, anchor links, and the table of contents.

use std::collections::HashSet;

use cuhlumn_api::wire::TocEntry;
use maud::html;
use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag, TagEnd};

use crate::render::markdown::slugify;

/// A heading whose content is still being read.
pub struct OpenHeading<'a> {
    level: HeadingLevel,
    id: Option<CowStr<'a>>,
    classes: Vec<CowStr<'a>>,
    attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)>,
    inner: Vec<Event<'a>>,
    label: String,
}

/// The ids that the document uses and its table of contents.
#[derive(Default)]
pub struct Headings {
    ids: HashSet<String>,
    toc: Vec<TocEntry>,
}

impl<'a> OpenHeading<'a> {
    pub fn new(
        level: HeadingLevel,
        id: Option<CowStr<'a>>,
        classes: Vec<CowStr<'a>>,
        attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)>,
    ) -> Self {
        Self {
            level,
            id,
            classes,
            attrs,
            inner: Vec::new(),
            label: String::new(),
        }
    }

    /// Adds an event inside the heading.
    pub fn push(&mut self, event: Event<'a>) {
        self.inner.push(event);
    }

    /// Adds text to the label of the heading.
    pub fn push_label(&mut self, text: &str) {
        self.label.push_str(text);
    }
}

impl Headings {
    /// Gives the heading a unique id and writes its events to `out`.
    ///
    /// An explicit `{#id}` wins over the slug of the text. H2 and H3 headings get an anchor link
    /// and an entry in the table of contents.
    pub fn close<'a>(&mut self, heading: OpenHeading<'a>, out: &mut Vec<Event<'a>>) {
        let OpenHeading {
            level,
            id,
            classes,
            attrs,
            inner,
            label,
        } = heading;
        let base = match id {
            Some(id) => id.into_string(),
            None => slugify(&label),
        };
        let id = self.unique(base);
        let listed = matches!(level, HeadingLevel::H2 | HeadingLevel::H3);
        let anchor = listed.then(|| {
            html! {
                a.heading-link href={ "#" (id) } aria-hidden="true" tabindex="-1" { "#" }
            }
            .into_string()
        });
        if listed {
            self.toc.push(TocEntry {
                level: level as u8,
                id: id.clone(),
                label: label.trim().to_owned(),
            });
        }
        out.push(Event::Start(Tag::Heading {
            level,
            id: Some(id.into()),
            classes,
            attrs,
        }));
        out.extend(inner);
        out.extend(anchor.map(|anchor| Event::InlineHtml(anchor.into())));
        out.push(Event::End(TagEnd::Heading(level)));
    }

    /// Returns the table of contents.
    pub fn into_toc(self) -> Vec<TocEntry> {
        self.toc
    }

    /// Returns `base`, or `base-2`, `base-3`, and so on when the document already uses it.
    fn unique(&mut self, base: String) -> String {
        if self.ids.insert(base.clone()) {
            return base;
        }
        let mut number = 2_u32;
        loop {
            let candidate = format!("{base}-{number}");
            if self.ids.insert(candidate.clone()) {
                return candidate;
            }
            number = number.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::render::markdown::{render, slugify};

    #[test]
    fn slugs_keep_letters_and_digits() {
        assert_eq!(slugify("Why one file?"), "why-one-file");
        assert_eq!(slugify("  Thé & Crème  "), "thé-crème");
        assert_eq!(slugify("!!!"), "section");
    }

    #[test]
    fn ids_are_escaped_in_attributes() {
        let doc = render("## Title {#a\"b}\n");
        assert!(!doc.html.contains("a\"b"), "{}", doc.html);
        assert!(doc.html.contains("href=\"#a&quot;b\""), "{}", doc.html);
    }
}
