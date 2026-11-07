//! The composition of a share card: a 1200×630 SVG with accent shapes.
//!
//! The top row holds the logo or an initial badge and the site title. The title and the subtitle
//! sit in the middle. A rule separates the bottom row with the host on the left and the meta text
//! on the right. The style sets the colours of a dark ground, a light ground, or an accent ground.
//! All styles use the same sizes and weights, so the text fitting is the same for each style.

use std::ops::RangeInclusive;

use cuhlumn_api::site::{accent::Accent, theme::CardStyle};

use crate::{
    og::{
        fit::{Lines, TextBox, measure, one_line},
        palette::CardPalette,
    },
    render::{escape, initial, style::Rgb},
};

/// What a share card shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    /// The site name. The top row shows it unless it is the same as the title.
    pub site: String,
    pub title: String,
    pub subtitle: String,
    pub host: String,
    /// The day and reading time of a post, or nothing.
    pub meta: String,
    pub accent: Accent,
    pub style: CardStyle,
    /// Draws the logo image in place of the initial badge.
    pub logo: bool,
}

/// The `href` of the logo image. The card renderer resolves it to the logo bytes.
pub const LOGO_HREF: &str = "logo";

const FONT: &str = "Inter, Arial, DejaVu Sans, sans-serif";

/// The left edge of all text, right of the accent band.
const LEFT: f64 = 92.0;
/// The right edge of the title, the subtitle, and the bottom row.
const RIGHT: f64 = 1104.0;
/// The right edge of the top row, clear of the accent ring.
const TOP_ROW_END: f64 = 980.0;
const BADGE_TOP: f64 = 72.0;
const BADGE: f64 = 64.0;
const BADGE_RADIUS: f64 = 16.0;
const SITE_SIZE: u16 = 30;
/// The area of the title and the subtitle. The block is centered in it.
const BLOCK_TOP: f64 = 190.0;
const BLOCK_BOTTOM: f64 = 496.0;
const BLOCK_GAP: f64 = 26.0;
const TITLE_SIZES: RangeInclusive<u16> = 48..=88;
const TITLE_LEADING: f64 = 1.1;
const SUBTITLE_SIZE: u16 = 30;
const SUBTITLE_LEADING: f64 = 1.4;
const SUBTITLE_LINES: f64 = 2.0;
const RULE_TOP: f64 = 522.0;
const FOOT_BASELINE: f64 = 574.0;
const FOOT_SIZE: u16 = 26;
const MARKER: f64 = 14.0;
/// The distance from the middle of a line box down to the baseline, as a multiple of the size.
const CAP_MIDDLE: f64 = 0.36;

impl Card {
    /// Returns the card as SVG.
    #[must_use]
    pub fn svg(&self) -> String {
        let palette = CardPalette::new(self.style, Rgb::from_accent(&self.accent));
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="1200" height="630" viewBox="0 0 1200 630" font-family="{FONT}">"#
        );
        svg.push_str(&ground(&palette));
        svg.push_str(&self.top_row(&palette));
        svg.push_str(&self.block(&palette));
        svg.push_str(&self.bottom_row(&palette));
        svg.push_str("</svg>");
        svg
    }

    fn top_row(&self, palette: &CardPalette) -> String {
        let mut row = if self.logo {
            format!(
                r#"<image x="{LEFT}" y="{BADGE_TOP}" width="{BADGE}" height="{BADGE}" preserveAspectRatio="xMidYMid meet" xlink:href="{LOGO_HREF}"/>"#
            )
        } else {
            let center = LEFT + BADGE / 2.0;
            let baseline = BADGE_TOP + BADGE / 2.0 + CAP_MIDDLE * 34.0;
            format!(
                r#"<rect x="{LEFT}" y="{BADGE_TOP}" width="{BADGE}" height="{BADGE}" rx="{BADGE_RADIUS}" fill="{fill}"/><text x="{center}" y="{baseline:.1}" text-anchor="middle" font-size="34" font-weight="800" fill="{ink}">{letter}</text>"#,
                fill = palette.badge.hex(),
                ink = palette.badge_ink.hex(),
                letter = escape(&initial(&self.site)),
            )
        };
        if self.site.trim() == self.title.trim() {
            return row;
        }
        let left = LEFT + BADGE + 22.0;
        let site = one_line(&self.site, SITE_SIZE, TOP_ROW_END - left);
        let baseline = BADGE_TOP + BADGE / 2.0 + CAP_MIDDLE * f64::from(SITE_SIZE);
        row.push_str(&format!(
            r#"<text x="{left}" y="{baseline:.1}" font-size="{SITE_SIZE}" font-weight="700" fill="{fill}">{site}</text>"#,
            fill = palette.text.hex(),
            site = escape(&site),
        ));
        row
    }

    /// Fits the subtitle first, then gives the title the rest of the area.
    fn block(&self, palette: &CardPalette) -> String {
        let width = RIGHT - LEFT;
        let subtitle_box = TextBox {
            width,
            height: SUBTITLE_LINES * f64::from(SUBTITLE_SIZE) * SUBTITLE_LEADING,
            leading: SUBTITLE_LEADING,
        };
        let subtitle = subtitle_box.fit(&self.subtitle, SUBTITLE_SIZE..=SUBTITLE_SIZE);
        let subtitle_height = subtitle.height(SUBTITLE_LEADING);
        let reserved = if subtitle.lines.is_empty() {
            0.0
        } else {
            subtitle_height + BLOCK_GAP
        };
        let title_box = TextBox {
            width,
            height: BLOCK_BOTTOM - BLOCK_TOP - reserved,
            leading: TITLE_LEADING,
        };
        let title = title_box.fit(&self.title, TITLE_SIZES);
        let title_height = title.height(TITLE_LEADING);
        let gap = if title.lines.is_empty() || subtitle.lines.is_empty() {
            0.0
        } else {
            BLOCK_GAP
        };
        let top =
            BLOCK_TOP + (BLOCK_BOTTOM - BLOCK_TOP - title_height - gap - subtitle_height) / 2.0;
        let mut block = lines_text(&title, top, TITLE_LEADING, "800", palette.text);
        block.push_str(&lines_text(
            &subtitle,
            top + title_height + gap,
            SUBTITLE_LEADING,
            "400",
            palette.soft,
        ));
        block
    }

    fn bottom_row(&self, palette: &CardPalette) -> String {
        let size = f64::from(FOOT_SIZE);
        let meta = one_line(&self.meta, FOOT_SIZE, (RIGHT - LEFT) / 2.0);
        let host_left = LEFT + MARKER + 14.0;
        let meta_room = if meta.is_empty() {
            0.0
        } else {
            measure(&meta, size) + 32.0
        };
        let host = one_line(&self.host, FOOT_SIZE, RIGHT - host_left - meta_room);
        let marker_top = FOOT_BASELINE - CAP_MIDDLE * size - MARKER / 2.0;
        format!(
            r#"<rect x="{LEFT}" y="{RULE_TOP}" width="{rule}" height="2" fill="{rule_fill}" fill-opacity="{rule_opacity}"/><rect x="{LEFT}" y="{marker_top:.1}" width="{MARKER}" height="{MARKER}" rx="3" fill="{marker}"/><text x="{host_left}" y="{FOOT_BASELINE}" font-size="{FOOT_SIZE}" font-weight="600" fill="{host_fill}">{host}</text><text x="{RIGHT}" y="{FOOT_BASELINE}" text-anchor="end" font-size="{FOOT_SIZE}" fill="{meta_fill}">{meta}</text>"#,
            rule = RIGHT - LEFT,
            rule_fill = palette.rule.hex(),
            rule_opacity = palette.rule_opacity,
            marker = palette.badge.hex(),
            host_fill = palette.soft.hex(),
            host = escape(&host),
            meta_fill = palette.muted.hex(),
            meta = escape(&meta),
        )
    }
}

/// Draws the ground: the fill, the band at the left edge, and the disc and the ring in the top right corner.
fn ground(palette: &CardPalette) -> String {
    format!(
        r#"<rect width="1200" height="630" fill="{ground}"/><g fill-opacity="{opacity}"><rect width="20" height="630" fill="{shape}"/><circle cx="1200" cy="0" r="156" fill="{shape}"/></g><circle cx="1200" cy="0" r="200" fill="none" stroke="{shape}" stroke-width="6" stroke-opacity="{ring}"/>"#,
        ground = palette.ground.hex(),
        shape = palette.shape.hex(),
        opacity = palette.shape_opacity,
        ring = palette.shape_opacity / 2.0,
    )
}

/// Returns lines as one `text` element whose first line box starts at `top`.
fn lines_text(lines: &Lines, top: f64, leading: f64, weight: &str, fill: Rgb) -> String {
    if lines.lines.is_empty() {
        return String::new();
    }
    let step = lines.size * leading;
    let mut text = format!(
        r#"<text font-size="{size}" font-weight="{weight}" fill="{fill}">"#,
        size = lines.size,
        fill = fill.hex(),
    );
    for (index, line) in lines.lines.iter().enumerate() {
        let baseline = top + step * (index as f64 + 0.5) + CAP_MIDDLE * lines.size;
        text.push_str(&format!(
            r#"<tspan x="{LEFT}" y="{baseline:.1}">{line}</tspan>"#,
            line = escape(line),
        ));
    }
    text.push_str("</text>");
    text
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::{accent::Accent, theme::CardStyle};
    use resvg::usvg;

    use crate::og::card::{Card, LOGO_HREF};

    fn card(title: &str) -> Card {
        Card {
            site: "Field Notes".to_owned(),
            title: title.to_owned(),
            subtitle: "Small lessons from running SQLite in production".to_owned(),
            host: "notes.example.com".to_owned(),
            meta: "March 3, 2026 · 6 min read".to_owned(),
            accent: Accent::new("#0f766e").unwrap(),
            style: CardStyle::Dark,
            logo: false,
        }
    }

    fn parses(svg: &str) {
        usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
    }

    #[test]
    fn text_is_escaped() {
        let svg = card("Tags like <b> & \"quotes\"").svg();
        for escaped in ["&lt;b&gt;", "&amp;", "&quot;quotes&quot;"] {
            assert!(svg.contains(escaped), "{escaped}");
        }
        parses(&svg);
    }

    #[test]
    fn the_logo_replaces_the_initial() {
        let plain = card("Backups").svg();
        assert!(plain.contains(">F</text>"));
        assert!(!plain.contains(LOGO_HREF));
        let branded = Card {
            logo: true,
            ..card("Backups")
        }
        .svg();
        assert!(branded.contains(&format!(r#"xlink:href="{LOGO_HREF}""#)));
        assert!(!branded.contains(">F</text>"));
        parses(&branded);
    }

    #[test]
    fn the_site_title_shows_once() {
        let post = card("Backups").svg();
        assert_eq!(post.matches("Field Notes").count(), 1);
        let site = card("Field Notes").svg();
        assert_eq!(site.matches("Field Notes").count(), 1);
    }

    #[test]
    fn every_style_has_the_same_layout() {
        // The parts that set where text goes and how large it is, without the colours.
        let layout = |svg: &str| -> Vec<String> {
            svg.split(['<', '>'])
                .filter(|part| part.starts_with("tspan") || part.starts_with("text "))
                .map(|part| part.split(" fill=").next().unwrap_or(part).to_owned())
                .collect()
        };
        let dark = card("How we back up SQLite").svg();
        for style in [CardStyle::Light, CardStyle::Accent] {
            let styled = Card {
                style,
                ..card("How we back up SQLite")
            };
            assert_eq!(layout(&styled.svg()), layout(&dark), "{style}");
            parses(&styled.svg());
        }
    }

    #[test]
    fn empty_cards_do_not_fail() {
        let empty = Card {
            site: String::new(),
            subtitle: String::new(),
            host: String::new(),
            meta: String::new(),
            accent: Accent::default(),
            style: CardStyle::Accent,
            ..card("")
        };
        parses(&empty.svg());
    }
}
