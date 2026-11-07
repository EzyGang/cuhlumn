//! The colors of the five palettes, each with a light and a dark variant.

use cuhlumn_api::site::theme::Palette;

use crate::render::style::Rgb;

/// The surface and text colors of one variant of a palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    pub bg: Rgb,
    pub surface: Rgb,
    pub surface_muted: Rgb,
    pub text: Rgb,
    pub text_muted: Rgb,
    /// The color of dividers. It carries meaning only in the contrast palette.
    pub border: Rgb,
    pub code_bg: Rgb,
    pub warning_bg: Rgb,
    pub warning_text: Rgb,
}

/// The light and the dark variant of a palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Variants {
    pub light: Colors,
    pub dark: Colors,
}

/// The minimum contrast of body text in every palette except contrast.
pub const TEXT_CONTRAST: f64 = 4.5;
/// The minimum contrast of body text in the contrast palette.
pub const HIGH_CONTRAST: f64 = 7.0;

const WARNING_LIGHT: (Rgb, Rgb) = (Rgb::new(251, 239, 199), Rgb::new(59, 42, 0));
const WARNING_DARK: (Rgb, Rgb) = (Rgb::new(59, 44, 8), Rgb::new(249, 227, 163));

/// Builds one variant from seven channel triples and the shared warning colors.
const fn colors(values: [[u8; 3]; 7], warning: (Rgb, Rgb)) -> Colors {
    const fn rgb(value: [u8; 3]) -> Rgb {
        Rgb::new(value[0], value[1], value[2])
    }
    Colors {
        bg: rgb(values[0]),
        surface: rgb(values[1]),
        surface_muted: rgb(values[2]),
        text: rgb(values[3]),
        text_muted: rgb(values[4]),
        border: rgb(values[5]),
        code_bg: rgb(values[6]),
        warning_bg: warning.0,
        warning_text: warning.1,
    }
}

/// The palette of the first release: off-white paper and a dark grey night.
pub const PAPER: Variants = Variants {
    light: colors(
        [
            [247, 247, 244],
            [255, 255, 255],
            [236, 235, 230],
            [27, 29, 33],
            [84, 88, 96],
            [220, 218, 211],
            [239, 238, 233],
        ],
        WARNING_LIGHT,
    ),
    dark: colors(
        [
            [18, 20, 23],
            [26, 29, 34],
            [37, 41, 49],
            [232, 230, 225],
            [163, 166, 173],
            [47, 52, 60],
            [30, 34, 39],
        ],
        WARNING_DARK,
    ),
};

/// Cool blue greys.
pub const SLATE: Variants = Variants {
    light: colors(
        [
            [245, 247, 250],
            [255, 255, 255],
            [230, 234, 240],
            [23, 32, 51],
            [74, 85, 104],
            [213, 219, 228],
            [236, 239, 244],
        ],
        WARNING_LIGHT,
    ),
    dark: colors(
        [
            [15, 20, 28],
            [22, 29, 40],
            [33, 42, 56],
            [227, 232, 239],
            [154, 165, 181],
            [43, 54, 70],
            [26, 34, 48],
        ],
        WARNING_DARK,
    ),
};

/// Warm cream and brown.
pub const SEPIA: Variants = Variants {
    light: colors(
        [
            [246, 239, 226],
            [251, 246, 236],
            [235, 225, 207],
            [43, 33, 24],
            [94, 80, 64],
            [220, 207, 184],
            [239, 229, 211],
        ],
        WARNING_LIGHT,
    ),
    dark: colors(
        [
            [26, 22, 18],
            [34, 29, 23],
            [46, 39, 32],
            [236, 227, 212],
            [176, 164, 143],
            [58, 49, 40],
            [38, 32, 25],
        ],
        WARNING_DARK,
    ),
};

/// Neutral greys without a tint.
pub const MONO: Variants = Variants {
    light: colors(
        [
            [255, 255, 255],
            [250, 250, 250],
            [237, 237, 237],
            [17, 17, 17],
            [85, 85, 85],
            [217, 217, 217],
            [242, 242, 242],
        ],
        WARNING_LIGHT,
    ),
    dark: colors(
        [
            [17, 17, 17],
            [25, 25, 25],
            [38, 38, 38],
            [237, 237, 237],
            [166, 166, 166],
            [51, 51, 51],
            [28, 28, 28],
        ],
        WARNING_DARK,
    ),
};

/// Black and white with strong dividers, for readers who need more contrast.
pub const CONTRAST: Variants = Variants {
    light: colors(
        [
            [255, 255, 255],
            [255, 255, 255],
            [240, 240, 240],
            [0, 0, 0],
            [51, 51, 51],
            [89, 89, 89],
            [242, 242, 242],
        ],
        WARNING_LIGHT,
    ),
    dark: colors(
        [
            [0, 0, 0],
            [10, 10, 10],
            [26, 26, 26],
            [255, 255, 255],
            [208, 208, 208],
            [138, 138, 138],
            [20, 20, 20],
        ],
        WARNING_DARK,
    ),
};

impl Colors {
    /// Returns the four backgrounds that text and links sit on.
    #[must_use]
    pub const fn backgrounds(self) -> [Rgb; 4] {
        [self.bg, self.surface, self.surface_muted, self.code_bg]
    }
}

/// Returns the colors of a palette.
#[must_use]
pub const fn variants(palette: Palette) -> Variants {
    match palette {
        Palette::Paper => PAPER,
        Palette::Slate => SLATE,
        Palette::Sepia => SEPIA,
        Palette::Mono => MONO,
        Palette::Contrast => CONTRAST,
    }
}

/// Returns the minimum contrast of text and links in a palette.
#[must_use]
pub const fn text_contrast(palette: Palette) -> f64 {
    match palette {
        Palette::Contrast => HIGH_CONTRAST,
        _ => TEXT_CONTRAST,
    }
}
