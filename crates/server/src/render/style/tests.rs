use cuhlumn_api::site::{
    accent::Accent,
    brand::Brand,
    theme::{Font, Palette, Width},
};

use crate::render::style::{
    BLACK, Rgb, WHITE,
    css::{DARK_CHOICE, DARK_SYSTEM, LinkColors, STATIC_CSS, site_css, stylesheet},
    palettes::{CONTRAST, Colors, HIGH_CONTRAST, PAPER, text_contrast, variants},
};

const ACCENTS: [&str; 7] = [
    "#f59e0b", "#ffff00", "#2563eb", "#000000", "#ffffff", "#0e7490", "#abc",
];

fn brand(palette: Palette, width: Width) -> Brand {
    Brand {
        accent: Accent::new("#0e7490").unwrap(),
        palette,
        width,
        ..Brand::default()
    }
}

fn each_variant() -> impl Iterator<Item = (Palette, &'static str, Colors)> {
    Palette::ALL.iter().flat_map(|palette| {
        let pair = variants(*palette);
        [
            (*palette, "light", pair.light),
            (*palette, "dark", pair.dark),
        ]
    })
}

#[test]
fn link_colors_meet_contrast_for_every_accent_and_palette() {
    for palette in Palette::ALL.iter().copied() {
        let (pair, target) = (variants(palette), text_contrast(palette));
        for value in ACCENTS {
            let links = LinkColors::for_accent(&Accent::new(value).unwrap(), palette);
            for background in pair.light.backgrounds() {
                let ratio = links.light.contrast(background);
                assert!(ratio >= target, "{palette} {value} light: {ratio}");
            }
            for background in pair.dark.backgrounds() {
                let ratio = links.dark.contrast(background);
                assert!(ratio >= target, "{palette} {value} dark: {ratio}");
            }
            let button = links.light.contrast(WHITE);
            assert!(button >= target, "{palette} {value} button: {button}");
        }
    }
}

#[test]
fn body_and_muted_text_read_on_every_background() {
    for (palette, name, colors) in each_variant() {
        let target = text_contrast(palette);
        for text in [colors.text, colors.text_muted] {
            for background in colors.backgrounds() {
                let ratio = text.contrast(background);
                assert!(
                    ratio >= target,
                    "{palette} {name} {text:?} on {background:?}: {ratio}"
                );
            }
        }
    }
    assert!((text_contrast(Palette::Contrast) - HIGH_CONTRAST).abs() < f64::EPSILON);
}

#[test]
fn warning_tone_reads_in_every_variant() {
    for (palette, name, colors) in each_variant() {
        let ratio = colors.warning_text.contrast(colors.warning_bg);
        assert!(ratio >= 4.5, "{palette} {name} warning: {ratio}");
    }
}

#[test]
fn contrast_palette_borders_are_visible() {
    for colors in [CONTRAST.light, CONTRAST.dark] {
        for background in [colors.bg, colors.surface] {
            let ratio = colors.border.contrast(background);
            assert!(ratio >= 3.0, "border on {background:?}: {ratio}");
        }
    }
}

#[test]
fn paper_keeps_the_first_release_colors() {
    let light = [
        PAPER.light.bg,
        PAPER.light.surface,
        PAPER.light.surface_muted,
        PAPER.light.text,
        PAPER.light.text_muted,
        PAPER.light.border,
        PAPER.light.code_bg,
    ];
    let dark = [
        PAPER.dark.bg,
        PAPER.dark.surface,
        PAPER.dark.surface_muted,
        PAPER.dark.text,
        PAPER.dark.text_muted,
        PAPER.dark.border,
        PAPER.dark.code_bg,
    ];
    let hex = |colors: [Rgb; 7]| colors.map(Rgb::hex).join(" ");
    assert_eq!(
        hex(light),
        "#f7f7f4 #ffffff #ecebe6 #1b1d21 #545860 #dcdad3 #efeee9"
    );
    assert_eq!(
        hex(dark),
        "#121417 #1a1d22 #252931 #e8e6e1 #a3a6ad #2f343c #1e2227"
    );
}

#[test]
fn site_css_puts_each_variant_under_its_selectors() {
    let css = site_css(&brand(Palette::Sepia, Width::Normal), &[]);
    let pair = variants(Palette::Sepia);
    let block = |selector: &str| {
        let start = css.find(selector).unwrap() + selector.len();
        css[start..start + css[start..].find('}').unwrap()].to_owned()
    };
    let light = block(":root{");
    assert!(light.contains(&format!("--color-bg:{};", pair.light.bg.hex())));
    assert!(light.contains("--color-accent:#0e7490;"));
    assert!(light.contains("--measure:42rem;"));
    assert!(light.ends_with("color-scheme:light"));
    for selector in [format!("{DARK_CHOICE}{{"), format!("{DARK_SYSTEM}{{")] {
        let dark = block(&selector);
        assert!(dark.contains(&format!("--color-bg:{};", pair.dark.bg.hex())));
        assert!(dark.contains(&format!("--color-text:{};", pair.dark.text.hex())));
        assert!(dark.ends_with("color-scheme:dark"));
    }
    assert!(css.contains(&format!(
        "@media (prefers-color-scheme:dark){{{DARK_SYSTEM}{{"
    )));
}

#[test]
fn site_css_sets_width_and_font() {
    let narrow = site_css(&brand(Palette::Paper, Width::Narrow), &[]);
    assert!(narrow.contains("--measure:38rem;"));
    assert!(!narrow.contains("--font-body"));
    let wide = Brand {
        font: Font::Sans,
        ..brand(Palette::Paper, Width::Wide)
    };
    let css = site_css(&wide, &[]);
    assert!(css.contains("--measure:52rem;"));
    assert!(css.contains(":root{--font-body:var(--font-ui)}"));
}

#[test]
fn stylesheet_starts_with_the_static_rules() {
    let css = stylesheet(&Brand::default(), &[]);
    assert!(css.starts_with(STATIC_CSS));
    assert!(
        !STATIC_CSS.contains("--color-bg:"),
        "palettes live in site_css"
    );
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
