//! The blog font: `@font-face` rules for a library family, and the font stacks that use it.

use cuhlumn_api::site::{brand::Brand, theme::Font};

use crate::fonts::faces::LibraryFace;

/// The path prefix of library font files on every blog host.
pub const FONTS_PATH: &str = "/_fonts/";

/// Returns the library faces of the brand family, in library order.
///
/// The list is empty when the brand has no family or when the library does not have it.
#[must_use]
pub fn family_faces(brand: &Brand, library: &[LibraryFace]) -> Vec<LibraryFace> {
    let Some(family) = brand.font_family.as_deref() else {
        return Vec::new();
    };
    library
        .iter()
        .filter(|face| face.has_family(family))
        .cloned()
        .collect()
}

/// Returns the regular face, which the page head preloads.
#[must_use]
pub fn regular(faces: &[LibraryFace]) -> Option<&LibraryFace> {
    faces.iter().find(|face| face.is_regular())
}

/// Returns the `@font-face` rules of `faces` and the font stacks with the brand family first.
///
/// Without faces, a sans brand uses the interface stack for post text, and a serif brand needs no rule.
#[must_use]
pub fn rules(brand: &Brand, faces: &[LibraryFace]) -> String {
    let fallback = match brand.font {
        Font::Serif => "var(--font-serif)",
        Font::Sans => "var(--font-sans)",
    };
    let Some(family) = brand.font_family.as_deref().filter(|_| !faces.is_empty()) else {
        return match brand.font {
            Font::Serif => String::new(),
            Font::Sans => ":root{--font-body:var(--font-ui)}\n".to_owned(),
        };
    };
    let name = css_string(family);
    let mut css = String::new();
    for face in faces {
        css.push_str(&format!(
            "@font-face{{font-family:{name};src:url({FONTS_PATH}{}) format(\"{}\");font-weight:{};font-style:{};font-display:swap}}\n",
            face.file,
            face.file.css_format(),
            face.weight,
            face.style.css(),
        ));
    }
    css.push_str(&format!(
        ":root{{--font-body:{name},{fallback};--font-ui:{name},{fallback}}}\n"
    ));
    css
}

/// Returns `value` as a double-quoted CSS string.
///
/// Quotes, backslashes, angle brackets, and control characters become hex escapes,
/// so the text can neither end the string nor the `style` element.
#[must_use]
pub fn css_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        if matches!(character, '"' | '\\' | '<' | '>') || character.is_control() {
            quoted.push_str(&format!("\\{:x} ", u32::from(character)));
        } else {
            quoted.push(character);
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::{brand::Brand, theme::Font};

    use crate::{
        fonts::{
            FontName,
            faces::{FaceStyle, LibraryFace},
        },
        render::style::font::{css_string, family_faces, regular, rules},
    };

    fn face(file: &str, family: &str, weight: u16, style: FaceStyle) -> LibraryFace {
        LibraryFace {
            file: FontName::new(file).unwrap(),
            families: vec![family.to_owned()],
            weight,
            style,
        }
    }

    fn library() -> Vec<LibraryFace> {
        vec![
            face("Lora-Regular-0a1b2c3d.ttf", "Lora", 400, FaceStyle::Normal),
            face("Lora-Italic-1a2b3c4d.otf", "Lora", 400, FaceStyle::Italic),
            face("Inter-Bold-2b3c4d5e.ttf", "Inter", 700, FaceStyle::Normal),
        ]
    }

    fn brand(family: Option<&str>, font: Font) -> Brand {
        Brand {
            font,
            font_family: family.map(str::to_owned),
            ..Brand::default()
        }
    }

    #[test]
    fn a_library_family_declares_its_faces() {
        let brand = brand(Some("Lora"), Font::Sans);
        let faces = family_faces(&brand, &library());
        assert_eq!(faces.len(), 2);
        let css = rules(&brand, &faces);
        assert!(css.contains(
            "@font-face{font-family:\"Lora\";src:url(/_fonts/Lora-Regular-0a1b2c3d.ttf) format(\"truetype\");font-weight:400;font-style:normal;font-display:swap}"
        ));
        assert!(css.contains("format(\"opentype\");font-weight:400;font-style:italic;"));
        assert!(css.contains(
            ":root{--font-body:\"Lora\",var(--font-sans);--font-ui:\"Lora\",var(--font-sans)}"
        ));
        assert!(!css.contains("Inter"));
        let preload = regular(&faces).map(|face| face.file.as_str());
        assert_eq!(preload, Some("Lora-Regular-0a1b2c3d.ttf"));
    }

    #[test]
    fn a_missing_family_falls_back_to_the_system_stack() {
        let serif = brand(Some("Gone"), Font::Serif);
        assert!(family_faces(&serif, &library()).is_empty());
        assert_eq!(rules(&serif, &[]), "");
        let sans = brand(Some("Gone"), Font::Sans);
        assert_eq!(rules(&sans, &[]), ":root{--font-body:var(--font-ui)}\n");
        assert!(family_faces(&brand(None, Font::Serif), &library()).is_empty());
    }

    #[test]
    fn family_names_are_escaped() {
        assert_eq!(css_string("Lora"), "\"Lora\"");
        assert_eq!(
            css_string("a\"b\\c</style>"),
            "\"a\\22 b\\5c c\\3c /style\\3e \""
        );
        assert_eq!(css_string("x\u{1}"), "\"x\\1 \"");
    }
}
