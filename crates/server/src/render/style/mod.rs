//! The colors of a blog: WCAG contrast, the palettes, and the stylesheet.

pub mod css;
pub mod palettes;
#[cfg(test)]
mod tests;

use cuhlumn_api::site::accent::Accent;

/// A color with red, green, and blue channels from 0 to 255.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb([f64; 3]);

pub const WHITE: Rgb = Rgb::new(255, 255, 255);
pub const BLACK: Rgb = Rgb::new(0, 0, 0);

const DEFAULT_ACCENT: Rgb = Rgb::new(37, 99, 235);
const MIX_STEPS: u8 = 40;

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

    /// Mixes the color toward `toward` just far enough to reach `target` contrast on every background.
    #[must_use]
    pub fn readable_on(self, backgrounds: &[Self], toward: Self, target: f64) -> Self {
        (0..=MIX_STEPS)
            .map(|step| self.mix(toward, f64::from(step) / f64::from(MIX_STEPS)))
            .find(|color| {
                backgrounds
                    .iter()
                    .all(|background| color.contrast(*background) >= target)
            })
            .unwrap_or(toward)
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
}
