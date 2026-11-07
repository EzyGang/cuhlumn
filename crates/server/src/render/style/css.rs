//! The stylesheet of a blog: the static rules and the rules that come from the site brand.

use cuhlumn_api::site::{
    accent::Accent,
    brand::Brand,
    theme::{Font, Palette, Width},
};

use crate::render::{
    code_theme::{self, Variant},
    style::{
        BLACK, Rgb, WHITE,
        palettes::{Colors, text_contrast, variants},
    },
};

/// The rules that are the same for every blog.
pub const STATIC_CSS: &str = include_str!("../../../assets/style.css");

/// The dark variant applies when the reader or the site chooses dark.
pub const DARK_CHOICE: &str = ":root[data-theme=dark]";
/// The dark variant also applies when the system is dark and nobody chose light.
pub const DARK_SYSTEM: &str = ":root:not([data-theme=light])";
const DARK_MEDIA: &str = "@media (prefers-color-scheme:dark)";

/// Links are a little above the minimum contrast, so rounding never drops below it.
const LINK_MARGIN: f64 = 0.1;

/// The link colors of the light and the dark variant. Buttons use the light color with white text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkColors {
    pub light: Rgb,
    pub dark: Rgb,
}

impl LinkColors {
    /// Returns link colors near the accent that read on every background of the palette.
    #[must_use]
    pub fn for_accent(accent: &Accent, palette: Palette) -> Self {
        let color = Rgb::from_accent(accent);
        let pair = variants(palette);
        let target = text_contrast(palette) + LINK_MARGIN;
        Self {
            light: color.readable_on(&pair.light.backgrounds(), BLACK, target),
            dark: color.readable_on(&pair.dark.backgrounds(), WHITE, target),
        }
    }
}

/// Returns the full stylesheet of a site: [`STATIC_CSS`] and then [`site_css`].
#[must_use]
pub fn stylesheet(brand: &Brand) -> String {
    let mut css = STATIC_CSS.to_owned();
    css.push_str(&site_css(brand));
    css
}

/// Returns the rules that come from the brand: palette, accent, width, body font, and code colors.
///
/// The light variant is the default.
/// The dark variant applies under [`DARK_CHOICE`], and under [`DARK_SYSTEM`] when the system is dark.
#[must_use]
pub fn site_css(brand: &Brand) -> String {
    let pair = variants(brand.palette);
    let links = LinkColors::for_accent(&brand.accent, brand.palette);
    let light = links.light.hex();
    let mut css = format!(
        ":root{{{}--color-link:{light};--color-accent:{};--color-button:{light};--color-on-button:{};--measure:{};color-scheme:light}}\n",
        variables(pair.light),
        brand.accent,
        WHITE.hex(),
        measure(brand.width),
    );
    if brand.font == Font::Sans {
        css.push_str(":root{--font-body:var(--font-ui)}\n");
    }
    css.push_str(&code_theme::rules(brand.code_theme, Variant::Light, ""));
    let dark = format!(
        "{}--color-link:{};color-scheme:dark",
        variables(pair.dark),
        links.dark.hex()
    );
    css.push_str(&format!("{DARK_CHOICE}{{{dark}}}\n"));
    css.push_str(&code_theme::rules(
        brand.code_theme,
        Variant::Dark,
        DARK_CHOICE,
    ));
    css.push_str(&format!("{DARK_MEDIA}{{{DARK_SYSTEM}{{{dark}}}\n"));
    css.push_str(&code_theme::rules(
        brand.code_theme,
        Variant::Dark,
        DARK_SYSTEM,
    ));
    css.push_str("}\n");
    css
}

/// Returns the content width of a width option.
#[must_use]
pub const fn measure(width: Width) -> &'static str {
    match width {
        Width::Narrow => "38rem",
        Width::Normal => "42rem",
        Width::Wide => "52rem",
    }
}

/// Returns the custom properties of one palette variant, each followed by a semicolon.
fn variables(colors: Colors) -> String {
    [
        ("bg", colors.bg),
        ("surface", colors.surface),
        ("surface-muted", colors.surface_muted),
        ("text", colors.text),
        ("text-muted", colors.text_muted),
        ("border", colors.border),
        ("code-bg", colors.code_bg),
        ("warning-bg", colors.warning_bg),
        ("warning-text", colors.warning_text),
    ]
    .iter()
    .map(|(name, color)| format!("--color-{name}:{};", color.hex()))
    .collect()
}
