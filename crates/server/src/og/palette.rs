//! The colors of share cards. Every text color reads on the dark ground with a contrast of at least 4.5.

use crate::render::style::{BLACK, Rgb, WHITE};

/// The background of the card.
pub const GROUND: Rgb = Rgb::new(17, 19, 24);
/// The rule above the bottom row.
pub const RULE: Rgb = Rgb::new(42, 46, 56);
/// The title and the site title.
pub const TEXT: Rgb = Rgb::new(244, 245, 247);
/// The subtitle and the host.
pub const SOFT: Rgb = Rgb::new(195, 200, 209);
/// The meta text.
pub const MUTED: Rgb = Rgb::new(154, 161, 173);

/// Returns white or black, whichever reads better on `background`.
#[must_use]
pub fn ink_on(background: Rgb) -> Rgb {
    if background.contrast(WHITE) >= background.contrast(BLACK) {
        WHITE
    } else {
        BLACK
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        og::palette::{GROUND, MUTED, SOFT, TEXT, ink_on},
        render::style::Rgb,
    };

    #[test]
    fn text_colors_read_on_the_ground() {
        for color in [TEXT, SOFT, MUTED] {
            let contrast = color.contrast(GROUND);
            assert!(contrast >= 4.5, "{} has {contrast:.2}", color.hex());
        }
    }

    #[test]
    fn ink_reads_on_any_accent() {
        for channel in 0..=255 {
            for color in [
                Rgb::new(channel, channel, channel),
                Rgb::new(channel, 255 - channel, 128),
                Rgb::new(255, channel, 0),
            ] {
                let contrast = ink_on(color).contrast(color);
                assert!(contrast >= 4.5, "{} has {contrast:.2}", color.hex());
            }
        }
    }
}
