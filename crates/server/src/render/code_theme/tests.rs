use cuhlumn_api::site::theme::CodeTheme;

use crate::render::{
    code_theme::{
        Variant, classes,
        color::{MIN_CONTRAST, readable, rgb, to_hsl},
        rules,
    },
    style::{BLACK, Rgb, WHITE},
};

const VARIANTS: [Variant; 2] = [Variant::Light, Variant::Dark];

/// Returns the colour after `property:` at the start of `declarations`, such as `color:#112233`.
fn value(declarations: &str, property: &str) -> Option<Rgb> {
    let start = declarations.find(&format!("{property}:#"))? + property.len() + 2;
    let digits = declarations.get(start..start + 6)?;
    let value = u32::from_str_radix(digits, 16).ok()?;
    Some(rgb(
        [value >> 16, value >> 8, value].map(|channel| (channel & 0xff) as u8)
    ))
}

/// Splits CSS into its selectors and declarations.
fn split(css: &str) -> Vec<(&str, &str)> {
    css.split_terminator('}')
        .map(|rule| rule.split_once('{').unwrap())
        .collect()
}

#[test]
fn every_token_reads_on_the_code_background() {
    for theme in CodeTheme::ALL
        .iter()
        .copied()
        .filter(|t| *t != CodeTheme::None)
    {
        for variant in VARIANTS {
            let css = rules(theme, variant, "");
            let rules = split(&css);
            let background = value(rules[0].1, "background").unwrap();
            let mut colors = 0;
            for (selector, declarations) in rules {
                let Some(color) = value(declarations, "color") else {
                    continue;
                };
                colors += 1;
                let contrast = color.contrast(background);
                assert!(
                    contrast >= MIN_CONTRAST,
                    "{theme} {variant:?} {selector} has {contrast:.2}"
                );
            }
            assert!(colors > 10, "{theme} {variant:?} has {colors} colours");
        }
    }
}

#[test]
fn variants_have_light_and_dark_backgrounds() {
    for theme in CodeTheme::ALL
        .iter()
        .copied()
        .filter(|t| *t != CodeTheme::None)
    {
        let light = value(&rules(theme, Variant::Light, ""), "background").unwrap();
        let dark = value(&rules(theme, Variant::Dark, ""), "background").unwrap();
        assert!(light.contrast(BLACK) > light.contrast(WHITE), "{theme}");
        assert!(dark.contrast(WHITE) > dark.contrast(BLACK), "{theme}");
    }
}

#[test]
fn none_has_no_rules() {
    for variant in VARIANTS {
        assert!(rules(CodeTheme::None, variant, ":root").is_empty());
    }
}

#[test]
fn every_selector_starts_with_the_scope() {
    let css = rules(
        CodeTheme::Github,
        Variant::Dark,
        ":root[data-theme=\"dark\"]",
    );
    for (selectors, _) in split(&css) {
        for selector in selectors.split(',') {
            assert!(
                selector.starts_with(":root[data-theme=\"dark\"] .post-body pre"),
                "{selector}"
            );
        }
    }
    let plain = rules(CodeTheme::Ocean, Variant::Light, "");
    assert!(plain.starts_with(".post-body pre{background:#"), "{plain}");
    assert!(plain.contains(".post-body pre .hl-comment"), "{plain}");
}

#[test]
fn scopes_become_prefixed_class_selectors() {
    assert_eq!(classes("keyword.control"), ".hl-keyword.hl-control");
    assert_eq!(classes("source.c++"), ".hl-source.hl-c\\2b \\2b ");
}

#[test]
fn readable_colors_keep_their_hue() {
    let pale = [250, 200, 120];
    let fixed = readable(pale, [255, 255, 255]);
    assert!(rgb(fixed).contrast(WHITE) >= MIN_CONTRAST);
    let (before, after) = (to_hsl(pale)[0], to_hsl(fixed)[0]);
    assert!((before - after).abs() < 0.02, "{before} {after}");
    let dark_blue = [20, 40, 120];
    assert!(rgb(readable(dark_blue, [0, 43, 54])).contrast(Rgb::new(0, 43, 54)) >= MIN_CONTRAST);
    assert_eq!(readable([0, 0, 0], [255, 255, 255]), [0, 0, 0]);
}
