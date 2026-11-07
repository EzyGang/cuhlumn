//! Colour helpers for code themes: alpha blending and lightness changes that keep the hue.

use syntect::highlighting::Color;

use crate::render::style::{BLACK, Rgb, WHITE};

/// The minimum contrast of code text against the code background.
pub const MIN_CONTRAST: f64 = 4.6;

/// The number of lightness steps between a colour and black or white.
const LIGHTNESS_STEPS: u16 = 200;

/// Changes the lightness of `color` until it reads on `background`. The hue and the saturation stay.
#[must_use]
pub fn readable(color: [u8; 3], background: [u8; 3]) -> [u8; 3] {
    let ground = rgb(background);
    if rgb(color).contrast(ground) >= MIN_CONTRAST {
        return color;
    }
    let (target, extreme) = if ground.contrast(WHITE) >= ground.contrast(BLACK) {
        (1.0, [255; 3])
    } else {
        (0.0, [0; 3])
    };
    let [hue, saturation, lightness] = to_hsl(color);
    (1..=LIGHTNESS_STEPS)
        .map(|step| {
            let amount = f64::from(step) / f64::from(LIGHTNESS_STEPS);
            from_hsl([hue, saturation, lightness + (target - lightness) * amount])
        })
        .find(|candidate| rgb(*candidate).contrast(ground) >= MIN_CONTRAST)
        .unwrap_or(extreme)
}

/// Draws a colour with its alpha over an opaque background.
#[must_use]
pub fn over(color: Color, background: [u8; 3]) -> [u8; 3] {
    let alpha = f64::from(color.a) / 255.0;
    let mut mixed = background;
    for (channel, value) in mixed.iter_mut().zip([color.r, color.g, color.b]) {
        let blended = f64::from(value) * alpha + f64::from(*channel) * (1.0 - alpha);
        *channel = blended.round().clamp(0.0, 255.0) as u8;
    }
    mixed
}

/// Returns the channels of a colour without its alpha.
#[must_use]
pub const fn opaque(color: Color) -> [u8; 3] {
    [color.r, color.g, color.b]
}

#[must_use]
pub const fn rgb([red, green, blue]: [u8; 3]) -> Rgb {
    Rgb::new(red, green, blue)
}

/// Returns the hue, the saturation, and the lightness of a colour, each from 0 to 1.
#[must_use]
pub fn to_hsl(color: [u8; 3]) -> [f64; 3] {
    let [red, green, blue] = color.map(|channel| f64::from(channel) / 255.0);
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.0;
    let delta = max - min;
    if delta <= f64::EPSILON {
        return [0.0, 0.0, lightness];
    }
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let sector = if max <= red {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if max <= green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    [sector / 6.0, saturation, lightness]
}

/// Returns the colour of a hue, a saturation, and a lightness, each from 0 to 1.
#[must_use]
pub fn from_hsl([hue, saturation, lightness]: [f64; 3]) -> [u8; 3] {
    let lightness = lightness.clamp(0.0, 1.0);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = hue.rem_euclid(1.0) * 6.0;
    let second = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let [red, green, blue] = match sector as u8 {
        0 => [chroma, second, 0.0],
        1 => [second, chroma, 0.0],
        2 => [0.0, chroma, second],
        3 => [0.0, second, chroma],
        4 => [second, 0.0, chroma],
        _ => [chroma, 0.0, second],
    };
    let offset = lightness - chroma / 2.0;
    [red, green, blue].map(|channel| ((channel + offset) * 255.0).round().clamp(0.0, 255.0) as u8)
}
