//! Markdown files with YAML front matter, the file format of `post push` and `post get --markdown`.

use chrono::{DateTime, NaiveDate};
use cuhlumn_api::post::{Post, PostInput, slug::Slug};
use serde::Deserialize;
use serde_json::{Map, Value};

/// A post read from a Markdown file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostFile {
    /// The slug from the front matter, if the file sets one.
    pub slug: Option<Slug>,
    pub input: PostInput,
}

/// A Markdown file that is not a valid post.
#[derive(Debug, thiserror::Error)]
pub enum FrontMatterError {
    #[error("the file must start with YAML front matter between two `---` lines")]
    Missing,
    #[error("the front matter has no closing `---` line")]
    Unclosed,
    #[error("the front matter is not valid YAML: {0}")]
    Yaml(#[from] serde_saphyr::Error),
    #[error("the front matter must be a mapping of fields, such as `title: My post`")]
    NotMapping,
    #[error("body: write the body after the front matter, not as a field")]
    BodyField,
    #[error("slug: {0}")]
    Slug(#[source] serde_json::Error),
    #[error(
        "published_at: {0:?} is not a time: use unix seconds, 2026-05-01, or 2026-05-01T09:30:00Z"
    )]
    Date(String),
    #[error("{0}")]
    Field(#[from] serde_path_to_error::Error<serde_json::Error>),
}

impl PostFile {
    /// Reads a post from a Markdown document that starts with YAML front matter.
    ///
    /// The front matter holds the fields of [`PostInput`] and an optional `slug`.
    /// The text after the front matter is the body.
    ///
    /// # Errors
    ///
    /// Returns [`FrontMatterError`] when the front matter is missing or breaks a field rule.
    pub fn parse(text: &str) -> Result<Self, FrontMatterError> {
        let (yaml, body) = split(text)?.ok_or(FrontMatterError::Missing)?;
        let mut fields = match serde_saphyr::from_str::<Value>(yaml)? {
            Value::Object(fields) => fields,
            Value::Null => Map::new(),
            _ => return Err(FrontMatterError::NotMapping),
        };
        if fields.contains_key("body") {
            return Err(FrontMatterError::BodyField);
        }
        let slug = match fields.remove("slug") {
            Some(value) => Option::<Slug>::deserialize(value).map_err(FrontMatterError::Slug)?,
            None => None,
        };
        if let Some(Value::String(text)) = fields.get("published_at") {
            let time = parse_time(text).ok_or_else(|| FrontMatterError::Date(text.clone()))?;
            fields.insert("published_at".to_owned(), Value::from(time));
        }
        fields.insert("body".to_owned(), Value::from(unix_lines(body)));
        let input = serde_path_to_error::deserialize(Value::Object(fields))?;
        Ok(Self { slug, input })
    }
}

/// Returns the body of a Markdown document, without its front matter and with `\n` line breaks.
///
/// # Errors
///
/// Returns [`FrontMatterError::Unclosed`] when the front matter has no closing line.
pub fn body(text: &str) -> Result<String, FrontMatterError> {
    Ok(match split(text)? {
        Some((_, body)) => unix_lines(body),
        None => unix_lines(text),
    })
}

/// Writes a stored post as a Markdown document that [`PostFile::parse`] reads back.
#[must_use]
pub fn to_markdown(post: &Post) -> String {
    let tags: Vec<&str> = post
        .tags
        .as_slice()
        .iter()
        .map(|tag| tag.as_str())
        .collect();
    let mut document = String::from("---\n");
    field(&mut document, "slug", post.slug.as_str());
    field(&mut document, "title", &quoted(&post.title));
    field(&mut document, "description", &quoted(&post.description));
    field(&mut document, "tags", &Value::from(tags).to_string());
    field(&mut document, "kind", post.kind.as_str());
    field(&mut document, "status", post.status.as_str());
    if let Some(cover) = &post.cover {
        field(&mut document, "cover", &quoted(cover.as_str()));
    }
    if let Some(og_image) = &post.og_image {
        field(&mut document, "og_image", &quoted(og_image.as_str()));
    }
    if let Some(show_cta) = post.show_cta {
        field(&mut document, "show_cta", &show_cta.to_string());
    }
    if let Some(published_at) = post.published_at {
        field(&mut document, "published_at", &published_at.to_string());
    }
    document.push_str("---\n\n");
    document.push_str(&post.body);
    if !post.body.ends_with('\n') {
        document.push('\n');
    }
    document
}

/// Splits a document into its front matter and its body.
///
/// Returns `None` when the document does not start with a `---` line.
fn split(text: &str) -> Result<Option<(&str, &str)>, FrontMatterError> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(first) = text.split_inclusive('\n').next() else {
        return Ok(None);
    };
    if !is_fence(first) {
        return Ok(None);
    }
    let rest = &text[first.len()..];
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if is_fence(line) {
            let body = &rest[offset + line.len()..];
            return Ok(Some((
                &rest[..offset],
                body.trim_start_matches(['\r', '\n']),
            )));
        }
        offset += line.len();
    }
    Err(FrontMatterError::Unclosed)
}

fn is_fence(line: &str) -> bool {
    line.trim_end() == "---"
}

/// Replaces Windows line breaks with `\n`, so stored bodies do not depend on the editor.
fn unix_lines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Parses an RFC 3339 time or a `YYYY-MM-DD` date at midnight UTC as unix seconds.
fn parse_time(text: &str) -> Option<i64> {
    match DateTime::parse_from_rfc3339(text) {
        Ok(time) => Some(time.timestamp()),
        Err(_) => Some(
            NaiveDate::parse_from_str(text, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)?
                .and_utc()
                .timestamp(),
        ),
    }
}

/// Returns `text` as a double-quoted YAML scalar. JSON string syntax is valid YAML.
fn quoted(text: &str) -> String {
    Value::from(text).to_string()
}

fn field(document: &mut String, name: &str, value: &str) {
    document.push_str(name);
    document.push_str(": ");
    document.push_str(value);
    document.push('\n');
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::post::{Kind, Post, Status, slug::Slug};

    use crate::front_matter::{FrontMatterError, PostFile, body, to_markdown};

    const POST: &str = "---\ntitle: How we back up SQLite\ntags: [Ops, ops, Databases]\nstatus: published\n---\n\nWe copy the file.\n";

    #[test]
    fn reads_fields_and_body() {
        let file = PostFile::parse(POST).unwrap();
        assert_eq!(file.slug, None);
        assert_eq!(file.input.title, "How we back up SQLite");
        assert_eq!(file.input.status, Status::Published);
        assert_eq!(file.input.kind, Kind::Post);
        assert_eq!(file.input.body, "We copy the file.\n");
        let tags: Vec<&str> = file
            .input
            .tags
            .as_slice()
            .iter()
            .map(|tag| tag.as_str())
            .collect();
        assert_eq!(tags, ["Ops", "Databases"]);
    }

    #[test]
    fn windows_line_endings_and_bom_work() {
        let text = "\u{feff}---\r\ntitle: Hi\r\nslug: hello\r\n---\r\n\r\nText.\r\n";
        let file = PostFile::parse(text).unwrap();
        assert_eq!(file.slug, Some(Slug::new("hello").unwrap()));
        assert_eq!(file.input.title, "Hi");
        assert_eq!(file.input.body, "Text.\n");
    }

    #[test]
    fn dates_become_unix_seconds() {
        for (date, seconds) in [
            ("2026-05-01", 1_777_593_600),
            ("2026-05-01T09:30:00Z", 1_777_627_800),
            ("2026-05-01T11:30:00+02:00", 1_777_627_800),
        ] {
            let text = format!("---\ntitle: Hi\npublished_at: \"{date}\"\n---\nText.");
            let file = PostFile::parse(&text).unwrap();
            assert_eq!(file.input.published_at, Some(seconds), "{date}");
        }
        let text = "---\ntitle: Hi\npublished_at: soon\n---\nText.";
        assert!(matches!(
            PostFile::parse(text),
            Err(FrontMatterError::Date(_))
        ));
    }

    #[test]
    fn invalid_documents_name_the_problem() {
        let cases = [
            ("Just text.", "front matter"),
            ("---\ntitle: Hi\nText.", "closing"),
            ("---\ntitle: Hi\nauthor: Jane\n---\nText.", "author"),
            ("---\ntitle: Hi\nbody: x\n---\nText.", "body"),
            ("---\ntitle: Hi\nslug: Not A Slug\n---\nText.", "slug"),
            ("---\ntitle: Hi\nstatus: live\n---\nText.", "status"),
            ("---\ndescription: x\n---\nText.", "title"),
            ("---\ntitle: Hi\n---\n\n", "body"),
            ("---\n- a\n- b\n---\nText.", "mapping"),
        ];
        for (text, expected) in cases {
            let message = PostFile::parse(text).unwrap_err().to_string();
            assert!(message.contains(expected), "{text:?} gave {message:?}");
        }
    }

    #[test]
    fn markdown_output_reads_back_as_the_same_post() {
        let post: Post = serde_json::from_value(serde_json::json!({
            "slug": "sqlite-backups",
            "title": "Quotes \" and: colons # here",
            "description": "Line one\nline two",
            "body": "# Heading\n\n---\n\nAfter a rule.",
            "tags": ["Ops", "Databases"],
            "kind": "page",
            "status": "published",
            "cover": "/media/ba7816bf8f01cfea414140de5dae2223.png",
            "og_image": null,
            "show_cta": false,
            "published_at": 1_777_593_600,
            "updated_at": 1_777_600_000,
            "url": "https://blog.example.com/sqlite-backups/"
        }))
        .unwrap();
        let file = PostFile::parse(&to_markdown(&post)).unwrap();
        assert_eq!(file.slug, Some(post.slug.clone()));
        let mut expected = cuhlumn_api::post::PostInput::from(post);
        expected.body.push('\n');
        assert_eq!(file.input, expected);
    }

    #[test]
    fn show_cta_is_optional() {
        let file = PostFile::parse(POST).unwrap();
        assert_eq!(file.input.show_cta, None);
        let text = "---\ntitle: Hi\nshow_cta: false\n---\nText.";
        assert_eq!(PostFile::parse(text).unwrap().input.show_cta, Some(false));
        let text = "---\ntitle: Hi\nshow_cta: maybe\n---\nText.";
        let message = PostFile::parse(text).unwrap_err().to_string();
        assert!(message.contains("show_cta"), "{message}");
    }

    #[test]
    fn body_without_front_matter_is_the_whole_text() {
        assert_eq!(body("Plain *text*.").unwrap(), "Plain *text*.");
        assert_eq!(body("---\ntitle: Hi\n---\n\nText.").unwrap(), "Text.");
        assert_eq!(body("One\r\nTwo\r\n").unwrap(), "One\nTwo\n");
    }
}
