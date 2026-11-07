//! The stylesheet of a blog: the static rules, the palette constants, and the colors that come from the site accent.

use cuhlumn_api::site::{Font, accent::Accent};

/// The rules that are the same for every blog.
pub const STATIC_CSS: &str = include_str!("../../assets/style.css");

/// A color with red, green, and blue channels from 0 to 255.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb([f64; 3]);

/// The link colors of the light and the dark theme. Buttons use the light color with white text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkColors {
    pub light: Rgb,
    pub dark: Rgb,
}

/// The surface and text colors of one theme. `style.css` uses the same values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub bg: Rgb,
    pub surface: Rgb,
    pub surface_muted: Rgb,
    pub text: Rgb,
    pub text_muted: Rgb,
}

pub const WHITE: Rgb = Rgb::new(255, 255, 255);
pub const BLACK: Rgb = Rgb::new(0, 0, 0);

/// The light theme.
pub const LIGHT: Palette = Palette {
    bg: Rgb::new(247, 247, 244),
    surface: Rgb::new(255, 255, 255),
    surface_muted: Rgb::new(236, 235, 230),
    text: Rgb::new(27, 29, 33),
    text_muted: Rgb::new(84, 88, 96),
};

/// The dark theme.
pub const DARK: Palette = Palette {
    bg: Rgb::new(18, 20, 23),
    surface: Rgb::new(26, 29, 34),
    surface_muted: Rgb::new(37, 41, 49),
    text: Rgb::new(232, 230, 225),
    text_muted: Rgb::new(163, 166, 173),
};

const DEFAULT_ACCENT: Rgb = Rgb::new(37, 99, 235);
/// The target contrast. It is a little above the WCAG AA value 4.5, so rounding never drops below it.
const TARGET_CONTRAST: f64 = 4.6;
const MIX_STEPS: u8 = 40;

/// Returns the accent rules of a site: `--color-accent`, `--color-link`, `--color-button`, `--color-on-button`, and the body font.
///
/// Put the result after [`STATIC_CSS`].
#[must_use]
pub fn site_css(accent: &Accent, font: Font) -> String {
    let links = LinkColors::for_accent(accent);
    let (light, dark, on_button) = (links.light.hex(), links.dark.hex(), WHITE.hex());
    let mut css = format!(
        ":root{{--color-accent:{accent};--color-link:{light};--color-button:{light};--color-on-button:{on_button}}}\n\
         :root[data-theme=dark]{{--color-link:{dark}}}\n\
         @media (prefers-color-scheme:dark){{:root:not([data-theme=light]){{--color-link:{dark}}}}}\n"
    );
    if font == Font::Sans {
        css.push_str(":root{--font-body:var(--font-ui)}\n");
    }
    css
}

impl Rgb {
    /// Returns a color from its red, green, and blue channels.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self([red as f64, green as f64, blue as f64])
    }

    /// Reads an accent color. `#rgb` doubles each digit.
    #[must_use]
    pub fn from_accent(accent: &Accent) -> Self {
        let digits = accent.as_str().trim_start_matches('#');
        let expanded: String = match digits.len() {
            3 => digits.chars().flat_map(|digit| [digit, digit]).collect(),
            _ => digits.to_owned(),
        };
        match u32::from_str_radix(&expanded, 16) {
            Ok(value) if expanded.len() == 6 => {
                Self([value >> 16, value >> 8, value].map(|channel| f64::from(channel & 0xff)))
            }
            _ => DEFAULT_ACCENT,
        }
    }

    /// Returns the WCAG contrast ratio of two colors, from 1 to 21.
    #[must_use]
    pub fn contrast(self, other: Self) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// Returns white or black, whichever reads better on this color.
    #[must_use]
    pub fn readable_text(self) -> Self {
        if self.contrast(WHITE) >= self.contrast(BLACK) {
            WHITE
        } else {
            BLACK
        }
    }

    /// Returns the color as `#rrggbb`.
    #[must_use]
    pub fn hex(self) -> String {
        let [r, g, b] = self
            .0
            .map(|channel| channel.round().clamp(0.0, 255.0) as u8);
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    fn luminance(self) -> f64 {
        let linear = |channel: f64| {
            let value = channel / 255.0;
            if value <= 0.039_28 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        let [r, g, b] = self.0.map(linear);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn mix(self, toward: Self, amount: f64) -> Self {
        let mut channels = self.0;
        for (channel, target) in channels.iter_mut().zip(toward.0) {
            *channel += (target - *channel) * amount;
        }
        Self(channels)
    }

    /// Mixes the color toward `toward` just far enough to read on every background.
    fn readable_on(self, backgrounds: &[Self], toward: Self) -> Self {
        (0..=MIX_STEPS)
            .map(|step| self.mix(toward, f64::from(step) / f64::from(MIX_STEPS)))
            .find(|color| {
                backgrounds
                    .iter()
                    .all(|background| color.contrast(*background) >= TARGET_CONTRAST)
            })
            .unwrap_or(toward)
    }
}

impl Palette {
    /// Returns the three backgrounds that text and links sit on.
    #[must_use]
    pub const fn backgrounds(self) -> [Rgb; 3] {
        [self.bg, self.surface, self.surface_muted]
    }
}

impl LinkColors {
    /// Returns link colors near the accent that read on the light and the dark backgrounds.
    #[must_use]
    pub fn for_accent(accent: &Accent) -> Self {
        let color = Rgb::from_accent(accent);
        Self {
            light: color.readable_on(&LIGHT.backgrounds(), BLACK),
            dark: color.readable_on(&DARK.backgrounds(), WHITE),
        }
    }
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::{Font, accent::Accent};

    use crate::render::style::{BLACK, DARK, LIGHT, LinkColors, Rgb, STATIC_CSS, WHITE, site_css};

    #[test]
    fn link_colors_meet_contrast_for_every_accent() {
        for value in [
            "#f59e0b", "#ffff00", "#2563eb", "#000000", "#ffffff", "#0e7490", "#abc",
        ] {
            let links = LinkColors::for_accent(&Accent::new(value).unwrap());
            let (light, dark) = (links.light, links.dark);
            for background in LIGHT.backgrounds() {
                assert!(light.contrast(background) >= 4.5, "{value} light {light:?}");
            }
            for background in DARK.backgrounds() {
                assert!(dark.contrast(background) >= 4.5, "{value} dark {dark:?}");
            }
            assert!(light.contrast(WHITE) >= 4.5, "{value} button {light:?}");
        }
    }

    #[test]
    fn theme_text_reads_on_every_background() {
        for (name, palette) in [("light", LIGHT), ("dark", DARK)] {
            for text in [palette.text, palette.text_muted] {
                for background in palette.backgrounds() {
                    let ratio = text.contrast(background);
                    assert!(ratio >= 4.5, "{name} {text:?} on {background:?}: {ratio}");
                }
            }
        }
    }

    #[test]
    fn stylesheet_uses_the_palette_constants() {
        for (palette, selector) in [(LIGHT, ":root{"), (DARK, ":root[data-theme=dark]{")] {
            let start = STATIC_CSS.find(selector).unwrap();
            let block = &STATIC_CSS[start..start + STATIC_CSS[start..].find('}').unwrap()];
            for (name, color) in [
                ("--color-bg", palette.bg),
                ("--color-surface", palette.surface),
                ("--color-surface-muted", palette.surface_muted),
                ("--color-text", palette.text),
                ("--color-text-muted", palette.text_muted),
            ] {
                let rule = format!("{name}:{};", color.hex());
                assert!(block.contains(&rule), "{selector} lacks {rule}");
            }
        }
    }

    #[test]
    fn readable_text_picks_the_stronger_contrast() {
        assert_eq!(Rgb::new(14, 116, 144).readable_text(), WHITE);
        assert_eq!(Rgb::new(250, 204, 21).readable_text(), BLACK);
    }

    #[test]
    fn short_accents_expand_and_hex_round_trips() {
        let color = Rgb::from_accent(&Accent::new("#abc").unwrap());
        assert_eq!(color.hex(), "#aabbcc");
        assert_eq!(
            Rgb::from_accent(&Accent::new("#7c3aed").unwrap()).hex(),
            "#7c3aed"
        );
        assert_eq!(Rgb::new(14, 116, 144).hex(), "#0e7490");
    }

    #[test]
    fn site_css_sets_accent_and_font() {
        let accent = Accent::new("#2563eb").unwrap();
        let serif = site_css(&accent, Font::Serif);
        assert!(serif.starts_with(":root{--color-accent:#2563eb;--color-link:#"));
        assert!(serif.contains(";--color-on-button:#ffffff}"));
        assert!(!serif.contains("--font-body"));
        assert!(site_css(&accent, Font::Sans).contains(":root{--font-body:var(--font-ui)}"));
    }
}
