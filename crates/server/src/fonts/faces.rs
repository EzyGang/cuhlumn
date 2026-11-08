//! The faces of the library fonts, which blog stylesheets declare with `@font-face`.

use std::path::Path;

use resvg::usvg::fontdb::{Database, Style};

use crate::fonts::FontName;

/// One face of a library font file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryFace {
    /// The file name in the library folder.
    pub file: FontName,
    /// The family names of the face, without repeats.
    pub families: Vec<String>,
    /// The weight class, from 1 to 1000. A variable font gives the weight of its default instance.
    pub weight: u16,
    pub style: FaceStyle,
}

/// The slant of a face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceStyle {
    Normal,
    Italic,
    Oblique,
}

impl FaceStyle {
    /// Returns the value of the CSS `font-style` property.
    #[must_use]
    pub const fn css(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Italic => "italic",
            Self::Oblique => "oblique",
        }
    }
}

impl LibraryFace {
    /// Returns `true` when `family` is one of the family names of the face.
    #[must_use]
    pub fn has_family(&self, family: &str) -> bool {
        self.families.iter().any(|name| name == family)
    }

    /// Returns `true` for the regular face: weight 400 and normal style.
    #[must_use]
    pub fn is_regular(&self) -> bool {
        self.weight == 400 && self.style == FaceStyle::Normal
    }
}

/// Reads the faces of the files `names` in the folder `root`, in the order of `names`.
///
/// A collection file gives only its first face, because a URL cannot select another face.
/// Names that are not library names and files that cannot be read are left out with a warning.
#[must_use]
pub fn load(root: &Path, names: &[String]) -> Vec<LibraryFace> {
    let mut faces = Vec::new();
    for name in names {
        let Ok(file) = FontName::new(name) else {
            tracing::warn!(%name, "a library record has an invalid file name");
            continue;
        };
        match std::fs::read(root.join(file.as_str())) {
            Ok(bytes) => faces.extend(parse(&file, bytes)),
            Err(err) => tracing::warn!(%name, error = %err, "a library font file cannot be read"),
        }
    }
    faces
}

/// Returns the first face of the font bytes, or `None` when the bytes have no readable face.
fn parse(file: &FontName, bytes: Vec<u8>) -> Option<LibraryFace> {
    let mut database = Database::new();
    database.load_font_data(bytes);
    let face = database.faces().find(|face| face.index == 0)?;
    let mut families: Vec<String> = Vec::with_capacity(face.families.len());
    for (family, _) in &face.families {
        if !families.contains(family) {
            families.push(family.clone());
        }
    }
    let style = match face.style {
        Style::Normal => FaceStyle::Normal,
        Style::Italic => FaceStyle::Italic,
        Style::Oblique => FaceStyle::Oblique,
    };
    Some(LibraryFace {
        file: file.clone(),
        families,
        weight: face.weight.0,
        style,
    })
}
