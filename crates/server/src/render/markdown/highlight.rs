//! Syntax highlighting of fenced code blocks with classed spans.
//!
//! Each span gets the scope atoms of its token as classes with the prefix
//! [`CLASS_PREFIX`], such as `hl-keyword hl-control`.
//! The site stylesheet gives the classes their colours.
//! Blocks without a language or with an unknown language keep the plain output of pulldown-cmark.

use std::sync::LazyLock;

use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

use crate::render::{code_theme::CLASS_PREFIX, escape};

/// A fenced code block with a known language whose text is still being read.
pub struct OpenCode {
    /// The first word of the info string, as pulldown-cmark writes it in the class.
    language: String,
    syntax: &'static SyntaxReference,
    code: String,
}

/// The grammars of every built-in language, loaded once.
static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed {
    prefix: CLASS_PREFIX,
};

impl OpenCode {
    /// Starts a code block with the info string `info`.
    ///
    /// Returns nothing when the block has no language, an unknown language, or plain text.
    #[must_use]
    pub fn start(info: &str) -> Option<Self> {
        let language = info.split(' ').next().unwrap_or_default();
        if language.is_empty() {
            return None;
        }
        let syntaxes = &*SYNTAXES;
        let syntax = syntaxes.find_syntax_by_token(language)?;
        if std::ptr::eq(syntax, syntaxes.find_syntax_plain_text()) {
            return None;
        }
        Some(Self {
            language: language.to_owned(),
            syntax,
            code: String::new(),
        })
    }

    /// Adds text to the code.
    pub fn push(&mut self, text: &str) {
        self.code.push_str(text);
    }

    /// Returns the block as `<pre><code>` with highlighted spans.
    ///
    /// When the grammar fails on the text, the block shows the escaped text without spans.
    #[must_use]
    pub fn html(&self) -> String {
        let body = self.highlighted().unwrap_or_else(|error| {
            tracing::warn!(language = %self.language, %error, "a code block cannot be highlighted");
            escape(&self.code)
        });
        format!(
            "<pre><code class=\"language-{}\">{body}</code></pre>\n",
            escape(&self.language),
        )
    }

    fn highlighted(&self) -> Result<String, syntect::Error> {
        let mut generator =
            ClassedHTMLGenerator::new_with_class_style(self.syntax, &SYNTAXES, CLASS_STYLE);
        for line in LinesWithEndings::from(&self.code) {
            generator.parse_html_for_line_which_includes_newline(line)?;
        }
        Ok(generator.finalize())
    }
}

#[cfg(test)]
mod tests {
    use crate::render::markdown::render;

    #[test]
    fn known_languages_get_classed_spans() {
        let doc = render("```rust\nfn backup() -> u8 { 1 }\n```\n");
        let html = &doc.html;
        assert!(
            html.starts_with(
                r#"<pre><code class="language-rust"><span class="hl-source hl-rust">"#
            ),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="hl-storage hl-type hl-function hl-rust">fn</span>"#),
            "{html}"
        );
        assert!(html.ends_with("</code></pre>\n"), "{html}");
        assert_eq!(doc.text, "fn backup() -> u8 { 1 }");
    }

    #[test]
    fn unknown_and_missing_languages_stay_plain() {
        let unknown = render("```harbor-config\nkeep <7> days & \"weeks\"\n```\n").html;
        assert_eq!(
            unknown,
            "<pre><code class=\"language-harbor-config\">keep &lt;7&gt; days &amp; \"weeks\"\n</code></pre>\n"
        );
        assert_eq!(
            render("```\nsqlite3 harbor.db\n```\n").html,
            "<pre><code>sqlite3 harbor.db\n</code></pre>\n"
        );
        assert_eq!(
            render("    sqlite3 harbor.db\n").html,
            "<pre><code>sqlite3 harbor.db\n</code></pre>\n"
        );
        assert!(!render("```text\nplain words\n```\n").html.contains("<span"));
    }

    #[test]
    fn script_tags_in_code_are_escaped() {
        let html = render("```html\n<script>alert(\"Mara\")</script>\n```\n").html;
        assert!(!html.contains("<script"), "{html}");
        assert!(
            html.contains(r#"&lt;</span><span class="hl-entity hl-name hl-tag hl-script hl-html">script</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="hl-text hl-html hl-basic">"#),
            "{html}"
        );
        let unknown = render("```harbor\n<script>alert(1)</script>\n```\n").html;
        assert!(
            unknown.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
            "{unknown}"
        );
    }

    #[test]
    fn code_blocks_inside_lists_are_highlighted() {
        let html = render("- Run:\n\n  ```sh\n  echo harbor\n  ```\n").html;
        assert!(html.contains(r#"<li>"#), "{html}");
        assert!(
            html.contains(r#"<pre><code class="language-sh"><span class="hl-source hl-shell"#),
            "{html}"
        );
    }
}
