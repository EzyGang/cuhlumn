//! The colours of share cards. Every text colour reads on the ground of its style with a contrast of at least 4.5.

use cuhlumn_api::site::theme::CardStyle;

use crate::render::style::{BLACK, Rgb, WHITE};

/// The colours of one card.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardPalette {
    /// The background of the card.
    pub ground: Rgb,
    /// The title and the site title.
    pub text: Rgb,
    /// The subtitle and the host.
    pub soft: Rgb,
    /// The meta text.
    pub muted: Rgb,
    /// The rule above the bottom row.
    pub rule: Rgb,
    pub rule_opacity: f64,
    /// The band at the left edge and the disc and the ring in the top right corner.
    pub shape: Rgb,
    pub shape_opacity: f64,
    /// The initial badge and the host marker.
    pub badge: Rgb,
    /// The initial on the badge.
    pub badge_ink: Rgb,
}

/// Returns white or black, whichever reads better on `background`.
#[must_use]
pub fn ink_on(background: Rgb) -> Rgb {
    if background.contrast(WHITE) >= background.contrast(BLACK) {
        WHITE
    } else {
        BLACK
    }
}

/// The colours of a card with a neutral ground and accent shapes.
struct Neutral {
    ground: Rgb,
    rule: Rgb,
    text: Rgb,
    soft: Rgb,
    muted: Rgb,
}

const DARK: Neutral = Neutral {
    ground: Rgb::new(17, 19, 24),
    rule: Rgb::new(42, 46, 56),
    text: Rgb::new(244, 245, 247),
    soft: Rgb::new(195, 200, 209),
    muted: Rgb::new(154, 161, 173),
};

const LIGHT: Neutral = Neutral {
    ground: Rgb::new(248, 248, 245),
    rule: Rgb::new(219, 222, 228),
    text: Rgb::new(21, 24, 31),
    soft: Rgb::new(61, 67, 79),
    muted: Rgb::new(88, 95, 108),
};

/// The opacity of the shapes on the accent ground, so they stay behind the text.
const ACCENT_SHAPE_OPACITY: f64 = 0.18;
const ACCENT_RULE_OPACITY: f64 = 0.35;

impl CardPalette {
    /// Returns the colours of `style` with the site accent.
    #[must_use]
    pub fn new(style: CardStyle, accent: Rgb) -> Self {
        match style {
            CardStyle::Dark => DARK.with_accent(accent),
            CardStyle::Light => LIGHT.with_accent(accent),
            CardStyle::Accent => {
                let ink = ink_on(accent);
                Self {
                    ground: accent,
                    text: ink,
                    soft: ink,
                    muted: ink,
                    rule: ink,
                    rule_opacity: ACCENT_RULE_OPACITY,
                    shape: ink,
                    shape_opacity: ACCENT_SHAPE_OPACITY,
                    badge: ink,
                    badge_ink: accent,
                }
            }
        }
    }
}

impl Neutral {
    fn with_accent(&self, accent: Rgb) -> CardPalette {
        CardPalette {
            ground: self.ground,
            text: self.text,
            soft: self.soft,
            muted: self.muted,
            rule: self.rule,
            rule_opacity: 1.0,
            shape: accent,
            shape_opacity: 1.0,
            badge: accent,
            badge_ink: ink_on(accent),
        }
    }
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::theme::CardStyle;

    use crate::{
        og::palette::{CardPalette, ink_on},
        render::style::Rgb,
    };

    fn accents() -> impl Iterator<Item = Rgb> {
        (0..=255).flat_map(|channel| {
            [
                Rgb::new(channel, channel, channel),
                Rgb::new(channel, 255 - channel, 128),
                Rgb::new(255, channel, 0),
                Rgb::new(14, 116, channel),
            ]
        })
    }

    #[test]
    fn text_reads_on_the_ground_of_every_style() {
        for style in CardStyle::ALL.iter().copied() {
            for accent in accents() {
                let palette = CardPalette::new(style, accent);
                for color in [palette.text, palette.soft, palette.muted] {
                    let contrast = color.contrast(palette.ground);
                    assert!(contrast >= 4.5, "{style} {} has {contrast:.2}", color.hex());
                }
                let badge = palette.badge_ink.contrast(palette.badge);
                assert!(
                    badge >= 4.5,
                    "{style} badge on {} has {badge:.2}",
                    accent.hex()
                );
            }
        }
    }

    #[test]
    fn ink_reads_on_any_accent() {
        for color in accents() {
            let contrast = ink_on(color).contrast(color);
            assert!(contrast >= 4.5, "{} has {contrast:.2}", color.hex());
        }
    }
}
