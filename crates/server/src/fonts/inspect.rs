//! Checks that uploaded bytes are a TTF, OTF, or TTC font and reads its names.

use resvg::usvg::fontdb::Database;
use sha2::{Digest, Sha256};

use crate::fonts::FontError;

/// The facts about a font file that the library stores.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inspected {
    /// The file name, such as `Inter-Regular-1a2b3c4d.ttf`.
    pub name: String,
    /// The family names of the faces, without repeats.
    pub families: Vec<String>,
}

const STEM_MAX: usize = 64;

/// Parses `bytes` as a font file.
///
/// # Errors
///
/// Returns [`FontError::NotAFont`] when the bytes are not a TTF, OTF, or TTC file with a readable face.
pub fn inspect(bytes: &[u8]) -> Result<Inspected, FontError> {
    let extension = extension(bytes)
        .ok_or_else(|| FontError::NotAFont("the file is not a TTF, OTF, or TTC font".to_owned()))?;
    let mut database = Database::new();
    database.load_font_data(bytes.to_vec());
    let mut families = Vec::new();
    let mut stem = None;
    for face in database.faces() {
        for (family, _) in &face.families {
            if !families.contains(family) {
                families.push(family.clone());
            }
        }
        if stem.is_none() && !face.post_script_name.is_empty() {
            stem = Some(face.post_script_name.clone());
        }
    }
    if families.is_empty() {
        return Err(FontError::NotAFont(
            "the file has no font face that the server can read".to_owned(),
        ));
    }
    let stem = sanitize(stem.as_deref().or(families.first().map(String::as_str)));
    let digest = hex::encode(Sha256::digest(bytes));
    let short = digest.get(..8).unwrap_or(&digest);
    Ok(Inspected {
        name: format!("{stem}-{short}.{extension}"),
        families,
    })
}

/// Returns the file extension that the signature of `bytes` names.
fn extension(bytes: &[u8]) -> Option<&'static str> {
    match bytes.get(..4)? {
        b"ttcf" => Some("ttc"),
        b"OTTO" => Some("otf"),
        [0, 1, 0, 0] | b"true" => Some("ttf"),
        _ => None,
    }
}

/// Keeps ASCII letters, digits, and hyphens of a font name, at most 64 characters.
fn sanitize(name: Option<&str>) -> String {
    let mut stem = String::new();
    for ch in name.unwrap_or_default().chars() {
        let ch = match ch.is_ascii_alphanumeric() {
            true => ch,
            false => '-',
        };
        if ch == '-' && (stem.is_empty() || stem.ends_with('-')) {
            continue;
        }
        stem.push(ch);
        if stem.len() == STEM_MAX {
            break;
        }
    }
    let stem = stem.trim_end_matches('-');
    match stem.is_empty() {
        true => "font".to_owned(),
        false => stem.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::fonts::{
        FontError,
        inspect::{extension, inspect, sanitize},
    };

    #[test]
    fn signatures_name_the_extension() {
        assert_eq!(extension(b"ttcf...."), Some("ttc"));
        assert_eq!(extension(b"OTTO...."), Some("otf"));
        assert_eq!(extension(&[0, 1, 0, 0, 9]), Some("ttf"));
        assert_eq!(extension(b"wOF2...."), None);
        assert_eq!(extension(b"ab"), None);
    }

    #[test]
    fn names_keep_safe_characters() {
        assert_eq!(sanitize(Some("Inter-Regular")), "Inter-Regular");
        assert_eq!(sanitize(Some("Noto Sans JP / Bold")), "Noto-Sans-JP-Bold");
        assert_eq!(sanitize(Some("..")), "font");
        assert_eq!(sanitize(None), "font");
        assert_eq!(sanitize(Some(&"a".repeat(100))).len(), 64);
    }

    #[test]
    fn other_bytes_are_not_fonts() {
        let png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";
        assert!(matches!(inspect(png), Err(FontError::NotAFont(_))));
        let fake = [0_u8, 1, 0, 0, 0, 0, 0, 0];
        assert!(matches!(inspect(&fake), Err(FontError::NotAFont(_))));
    }
}
