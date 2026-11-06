//! Text fitting for share cards: estimated line widths, word wrapping, and the choice of a type size.
//!
//! The card cannot measure glyphs before resvg draws them, so widths come from the advance of each
//! character as a fraction of the type size. The factors follow the bold weight of the card font and
//! round up a little, so the drawn text is not wider than the estimate.

use std::{mem, ops::RangeInclusive};

/// An area that lines of text fill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextBox {
    /// The widest line, in pixels.
    pub width: f64,
    /// The height of all lines together, in pixels.
    pub height: f64,
    /// The distance between two baselines as a multiple of the type size.
    pub leading: f64,
}

/// Text broken into lines at one type size.
#[derive(Clone, Debug, PartialEq)]
pub struct Lines {
    pub size: f64,
    pub lines: Vec<String>,
}

/// The difference between two tried type sizes, in pixels.
const SIZE_STEP: usize = 2;
const ELLIPSIS: char = '…';

impl TextBox {
    /// Breaks `text` into lines at the largest size in `sizes` at which every line fits the box.
    ///
    /// When no size fits, the text gets the smallest size and its last line ends with an ellipsis.
    #[must_use]
    pub fn fit(self, text: &str, sizes: RangeInclusive<u16>) -> Lines {
        let smallest = f64::from(*sizes.start());
        for size in sizes.rev().step_by(SIZE_STEP).map(f64::from) {
            let lines = wrap(text, size, self.width);
            if lines.len() <= self.max_lines(size) {
                return Lines { size, lines };
            }
        }
        let lines = wrap(text, smallest, self.width);
        let lines = cut(lines, self.max_lines(smallest), smallest, self.width);
        Lines {
            size: smallest,
            lines,
        }
    }

    /// Returns how many lines of `size` the box holds.
    #[must_use]
    pub fn max_lines(self, size: f64) -> usize {
        let step = size * self.leading;
        if step <= 0.0 {
            return 0;
        }
        (self.height / step).floor() as usize
    }
}

impl Lines {
    /// Returns the height of the lines, in pixels, for `leading` as a multiple of the size.
    #[must_use]
    pub fn height(&self, leading: f64) -> f64 {
        self.lines.len() as f64 * self.size * leading
    }
}

/// Returns the part of `text` that fits one line of `width` at `size`, with an ellipsis when it is cut.
#[must_use]
pub fn one_line(text: &str, size: u16, width: f64) -> String {
    let line = TextBox {
        width,
        height: f64::from(size),
        leading: 1.0,
    };
    line.fit(text, size..=size)
        .lines
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// Returns the estimated width of `text` at `size`, in pixels.
#[must_use]
pub fn measure(text: &str, size: f64) -> f64 {
    text.chars().map(advance).sum::<f64>() * size
}

/// Returns the estimated advance of a character as a fraction of the type size.
fn advance(character: char) -> f64 {
    match character {
        ' ' => 0.28,
        'i' | 'j' | 'l' | 'I' | '.' | ',' | ':' | ';' | '\'' | '!' | '|' => 0.31,
        'f' | 't' | 'r' | '(' | ')' | '[' | ']' | '"' | '-' | '/' | '’' | '‘' => 0.43,
        'm' | 'w' => 0.92,
        'M' | 'W' | '@' | '%' | '—' | ELLIPSIS => 1.0,
        'A'..='Z' => 0.75,
        'a'..='z' | '0'..='9' => 0.63,
        _ if is_wide(character) => 1.0,
        _ => 0.68,
    }
}

/// Returns true for full-width scripts and emoji, which take a full em.
fn is_wide(character: char) -> bool {
    matches!(
        character,
        '\u{1100}'..='\u{115f}'
            | '\u{2e80}'..='\u{a4cf}'
            | '\u{ac00}'..='\u{d7a3}'
            | '\u{f900}'..='\u{faff}'
            | '\u{fe30}'..='\u{fe4f}'
            | '\u{ff00}'..='\u{ff60}'
            | '\u{1f000}'..
    )
}

/// Breaks text into lines no wider than `width` at word boundaries.
///
/// A word that is wider than a line breaks between characters.
fn wrap(text: &str, size: f64, width: f64) -> Vec<String> {
    let space = advance(' ') * size;
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0.0;
    for word in text.split_whitespace() {
        let word_width = measure(word, size);
        if !line.is_empty() && used + space + word_width <= width {
            line.push(' ');
            line.push_str(word);
            used += space + word_width;
            continue;
        }
        if !line.is_empty() {
            lines.push(mem::take(&mut line));
            used = 0.0;
        }
        for character in word.chars() {
            let character_width = advance(character) * size;
            if !line.is_empty() && used + character_width > width {
                lines.push(mem::take(&mut line));
                used = 0.0;
            }
            line.push(character);
            used += character_width;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Keeps the first `max` lines. When lines drop, the last kept line ends with an ellipsis.
fn cut(mut lines: Vec<String>, max: usize, size: f64, width: f64) -> Vec<String> {
    if lines.len() <= max {
        return lines;
    }
    lines.truncate(max);
    if let Some(last) = lines.last_mut() {
        *last = with_ellipsis(last, size, width);
    }
    lines
}

/// Shortens a line so that it fits `width` with an ellipsis at the end.
fn with_ellipsis(line: &str, size: f64, width: f64) -> String {
    let room = width - advance(ELLIPSIS) * size;
    let mut used = 0.0;
    let end = line
        .char_indices()
        .find(|&(_, character)| {
            used += advance(character) * size;
            used > room
        })
        .map_or(line.len(), |(index, _)| index);
    let mut kept = &line[..end];
    if end < line.len() {
        kept = kept.rfind(' ').map_or(kept, |space| &kept[..space]);
    }
    let mut shortened = kept
        .trim_end_matches(|character: char| !character.is_alphanumeric())
        .to_owned();
    shortened.push(ELLIPSIS);
    shortened
}

#[cfg(test)]
mod tests {
    use crate::og::fit::{TextBox, measure, one_line, wrap};

    const TITLE: TextBox = TextBox {
        width: 1000.0,
        height: 300.0,
        leading: 1.1,
    };

    fn assert_fits(text_box: TextBox, text: &str) {
        let fitted = text_box.fit(text, 40..=90);
        assert!(fitted.height(text_box.leading) <= text_box.height, "{text}");
        for line in &fitted.lines {
            let width = measure(line, fitted.size);
            assert!(width <= text_box.width, "{line:?} is {width} wide");
        }
    }

    #[test]
    fn short_titles_get_the_largest_size() {
        let fitted = TITLE.fit("Hello", 40..=90);
        assert!((fitted.size - 90.0).abs() < f64::EPSILON);
        assert_eq!(fitted.lines, ["Hello"]);
    }

    #[test]
    fn longer_titles_get_smaller_sizes() {
        let short = TITLE.fit("Backups for SQLite", 40..=90);
        let long = TITLE.fit(
            "How we back up a busy SQLite database every five minutes without stopping writes",
            40..=90,
        );
        assert!(long.size < short.size, "{} < {}", long.size, short.size);
        assert!(long.lines.len() > 1);
    }

    #[test]
    fn every_line_fits_the_box() {
        for text in [
            "A",
            "Backups for SQLite",
            "Supercalifragilisticexpialidociousness and other unbreakably long words",
            "WWWWWWWWWW MMMMMMMMMM WWWWWWWWWW MMMMMMMMMM WWWWWWWWWW MMMMMMMMMM",
            "日本語のタイトルはとても長くなることがありますが、カードの中に収まる必要があります",
            &"word ".repeat(200),
        ] {
            assert_fits(TITLE, text);
        }
    }

    #[test]
    fn overflowing_text_ends_with_an_ellipsis() {
        let fitted = TITLE.fit(&"word ".repeat(200), 40..=90);
        assert!((fitted.size - 40.0).abs() < f64::EPSILON);
        assert_eq!(fitted.lines.len(), TITLE.max_lines(40.0));
        assert!(
            fitted
                .lines
                .iter()
                .rev()
                .skip(1)
                .all(|line| !line.ends_with('…'))
        );
        assert!(fitted.lines.last().unwrap().ends_with("word…"));
    }

    #[test]
    fn empty_text_and_empty_boxes_do_not_fail() {
        assert!(TITLE.fit("", 40..=90).lines.is_empty());
        assert!(TITLE.fit("   ", 40..=90).lines.is_empty());
        let flat = TextBox {
            height: 0.0,
            ..TITLE
        };
        assert!(flat.fit("Some title", 40..=90).lines.is_empty());
        let narrow = TextBox {
            width: 0.0,
            ..TITLE
        };
        assert!(!narrow.fit("Some title", 40..=90).lines.is_empty());
    }

    #[test]
    fn words_wrap_at_spaces() {
        assert_eq!(wrap("one two three", 10.0, 60.0), ["one two", "three"]);
    }

    #[test]
    fn one_line_keeps_short_text_and_cuts_long_text() {
        assert_eq!(
            one_line("notes.example.com", 26, 600.0),
            "notes.example.com"
        );
        let cut = one_line("The Quiet Engineering Weekly Journal", 30, 300.0);
        assert!(cut.ends_with('…'), "{cut}");
        assert!(measure(&cut, 30.0) <= 300.0, "{cut}");
    }
}
