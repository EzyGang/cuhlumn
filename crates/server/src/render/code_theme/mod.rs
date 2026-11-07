//! The colours of highlighted code: CSS rules for the classed spans that `render::markdown` writes.
//!
//! Every code theme uses two built-in syntect themes, one for each variant:
//!
//! | Code theme | Light variant | Dark variant |
//! |---|---|---|
//! | `github` | `InspiredGitHub` | `base16-eighties.dark` |
//! | `solarized` | `Solarized (light)` | `Solarized (dark)` |
//! | `ocean` | `base16-ocean.light` | `base16-ocean.dark` |
//! | `none` | no rules | no rules |
//!
//! The rules keep the hue and the saturation of each token colour.
//! They change its lightness until it has a contrast of at least 4.6 against the code background.
//! The rules do not use the token backgrounds of the themes.

mod color;
#[cfg(test)]
mod tests;

use std::sync::LazyLock;

use cuhlumn_api::site::theme::CodeTheme;
use syntect::highlighting::{FontStyle, Theme, ThemeItem, ThemeSet};

use crate::render::{
    code_theme::color::{opaque, over, readable, rgb},
    style::Rgb,
};

/// The prefix of every class in highlighted code.
pub const CLASS_PREFIX: &str = "hl-";

/// The colour variant of a code theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Light,
    Dark,
}

/// Returns the CSS rules of `theme` in `variant`.
///
/// The rules set the background and the colour of `.post-body pre` and the colours of the token spans.
/// When `scope` is not empty, every selector starts with `scope` and a space.
/// [`CodeTheme::None`] gives an empty string.
#[must_use]
pub fn rules(theme: CodeTheme, variant: Variant, scope: &str) -> String {
    let Some(compiled) = Compiled::get(theme, variant) else {
        return String::new();
    };
    let base = if scope.is_empty() {
        ".post-body pre".to_owned()
    } else {
        format!("{scope} .post-body pre")
    };
    let mut css = base.clone();
    css.push_str("{background:");
    css.push_str(&compiled.background.hex());
    css.push_str(";color:");
    css.push_str(&compiled.foreground.hex());
    css.push('}');
    for rule in &compiled.rules {
        for (index, selector) in rule.selectors.iter().enumerate() {
            if index > 0 {
                css.push(',');
            }
            css.push_str(&base);
            css.push(' ');
            css.push_str(selector);
        }
        css.push('{');
        css.push_str(&rule.declarations);
        css.push('}');
    }
    css
}

const LIGHT_FALLBACK: [u8; 3] = [255, 255, 255];
const DARK_FALLBACK: [u8; 3] = [30, 30, 30];
/// The syntect themes of each code theme, light variant first.
const THEME_NAMES: [[&str; 2]; 3] = [
    ["InspiredGitHub", "base16-eighties.dark"],
    ["Solarized (light)", "Solarized (dark)"],
    ["base16-ocean.light", "base16-ocean.dark"],
];

/// The compiled colours of every code theme in both variants, in the order of [`THEME_NAMES`].
static COMPILED: LazyLock<[[Option<Compiled>; 2]; 3]> = LazyLock::new(|| {
    let themes = ThemeSet::load_defaults();
    THEME_NAMES.map(|names| {
        [Variant::Light, Variant::Dark]
            .map(|variant| Compiled::load(&themes, names[variant.index()], variant))
    })
});

/// The colours of one code theme in one variant, with every colour made readable.
struct Compiled {
    background: Rgb,
    foreground: Rgb,
    rules: Vec<Rule>,
}

/// One CSS rule without the scope and the `.post-body pre` part of its selectors.
struct Rule {
    selectors: Vec<String>,
    declarations: String,
}

impl Variant {
    const fn index(self) -> usize {
        match self {
            Self::Light => 0,
            Self::Dark => 1,
        }
    }
}

impl Compiled {
    /// Returns the compiled colours of `theme` in `variant`.
    fn get(theme: CodeTheme, variant: Variant) -> Option<&'static Self> {
        let index = match theme {
            CodeTheme::Github => 0,
            CodeTheme::Solarized => 1,
            CodeTheme::Ocean => 2,
            CodeTheme::None => return None,
        };
        COMPILED[index][variant.index()].as_ref()
    }

    fn load(themes: &ThemeSet, name: &str, variant: Variant) -> Option<Self> {
        let Some(source) = themes.themes.get(name) else {
            tracing::warn!(name, "the built-in code theme is missing");
            return None;
        };
        Some(Self::build(source, variant))
    }

    fn build(source: &Theme, variant: Variant) -> Self {
        let fallback = match variant {
            Variant::Light => LIGHT_FALLBACK,
            Variant::Dark => DARK_FALLBACK,
        };
        let background = source.settings.background.map_or(fallback, opaque);
        let foreground = source
            .settings
            .foreground
            .map_or([128; 3], |color| over(color, background));
        Self {
            background: rgb(background),
            foreground: rgb(readable(foreground, background)),
            rules: source
                .scopes
                .iter()
                .filter_map(|item| Rule::build(item, background))
                .collect(),
        }
    }
}

impl Rule {
    /// Returns the rule of a theme item, or nothing when the item sets no colour or font style.
    fn build(item: &ThemeItem, background: [u8; 3]) -> Option<Self> {
        let mut declarations = String::new();
        if let Some(color) = item.style.foreground {
            let color = rgb(readable(over(color, background), background));
            declarations = format!("color:{};", color.hex());
        }
        if let Some(style) = item.style.font_style {
            for (flag, declaration) in [
                (FontStyle::BOLD, "font-weight:bold;"),
                (FontStyle::ITALIC, "font-style:italic;"),
                (FontStyle::UNDERLINE, "text-decoration:underline;"),
            ] {
                if style.contains(flag) {
                    declarations.push_str(declaration);
                }
            }
        }
        let selectors: Vec<String> = item
            .scope
            .selectors
            .iter()
            .map(|selector| {
                let parts: Vec<String> = selector
                    .extract_scopes()
                    .iter()
                    .map(|scope| classes(&scope.build_string()))
                    .collect();
                parts.join(" ")
            })
            .filter(|selector| !selector.is_empty())
            .collect();
        if declarations.is_empty() || selectors.is_empty() {
            return None;
        }
        Some(Self {
            selectors,
            declarations,
        })
    }
}

/// Returns a scope such as `keyword.control` as the class selector `.hl-keyword.hl-control`.
fn classes(scope: &str) -> String {
    let mut selector = String::new();
    for atom in scope.split('.').filter(|atom| !atom.is_empty()) {
        selector.push('.');
        selector.push_str(CLASS_PREFIX);
        for character in atom.chars() {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                selector.push(character);
            } else {
                selector.push_str(&format!("\\{:x} ", u32::from(character)));
            }
        }
    }
    selector
}
