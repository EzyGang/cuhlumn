//! Image formats that the server accepts.

/// An image format that the server accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
    Avif,
}

const ALL: [ImageFormat; 5] = [
    ImageFormat::Png,
    ImageFormat::Jpeg,
    ImageFormat::Webp,
    ImageFormat::Gif,
    ImageFormat::Avif,
];

impl ImageFormat {
    /// Reads a `Content-Type` value and ignores its parameters.
    #[must_use]
    pub fn from_content_type(value: &str) -> Option<Self> {
        let essence = value.split(';').next().unwrap_or_default().trim();
        ALL.into_iter()
            .find(|format| essence.eq_ignore_ascii_case(format.content_type()))
    }

    /// Reads a file extension without the dot.
    #[must_use]
    pub fn from_extension(value: &str) -> Option<Self> {
        ALL.into_iter().find(|format| format.extension() == value)
    }

    /// Returns the MIME type, such as `image/png`.
    #[must_use]
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Gif => "image/gif",
            Self::Avif => "image/avif",
        }
    }

    /// Returns the file extension without the dot.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Gif => "gif",
            Self::Avif => "avif",
        }
    }

    /// Returns `true` when the bytes start with the signature of this format.
    #[must_use]
    pub fn matches(self, bytes: &[u8]) -> bool {
        match self {
            Self::Png => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            Self::Jpeg => bytes.starts_with(&[0xff, 0xd8, 0xff]),
            Self::Gif => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
            Self::Webp => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
            Self::Avif => is_avif(bytes),
        }
    }
}

/// Checks for an ISO BMFF `ftyp` box with the `avif` or `avis` brand.
fn is_avif(bytes: &[u8]) -> bool {
    if bytes.get(4..8) != Some(b"ftyp") {
        return false;
    }
    let declared = bytes
        .get(..4)
        .and_then(|size| <[u8; 4]>::try_from(size).ok())
        .map_or(0, u32::from_be_bytes);
    let end = usize::try_from(declared).map_or(bytes.len(), |size| size.min(bytes.len()));
    let brands = bytes.get(8..end).unwrap_or_default();
    brands
        .chunks_exact(4)
        .enumerate()
        .filter(|(index, _)| *index != 1)
        .any(|(_, brand)| brand == b"avif" || brand == b"avis")
}

#[cfg(test)]
mod tests {
    use crate::media::format::ImageFormat;

    const AVIF: &[u8] = b"\0\0\0\x1cftypavif\0\0\0\0avifmif1miaf";
    const AVIF_COMPATIBLE: &[u8] = b"\0\0\0\x18ftypmif1\0\0\0\0avifmiaf";

    #[test]
    fn content_type_ignores_parameters_and_case() {
        assert_eq!(
            ImageFormat::from_content_type("image/png"),
            Some(ImageFormat::Png)
        );
        assert_eq!(
            ImageFormat::from_content_type("Image/JPEG; charset=binary"),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(ImageFormat::from_content_type("image/svg+xml"), None);
        assert_eq!(ImageFormat::from_content_type(""), None);
    }

    #[test]
    fn signatures_match_their_format() {
        assert!(ImageFormat::Png.matches(b"\x89PNG\r\n\x1a\nrest"));
        assert!(ImageFormat::Jpeg.matches(&[0xff, 0xd8, 0xff, 0xe0]));
        assert!(ImageFormat::Gif.matches(b"GIF89a..."));
        assert!(ImageFormat::Webp.matches(b"RIFF\0\0\0\0WEBPVP8 "));
        assert!(ImageFormat::Avif.matches(AVIF));
        assert!(ImageFormat::Avif.matches(AVIF_COMPATIBLE));
    }

    #[test]
    fn signatures_reject_other_bytes() {
        assert!(!ImageFormat::Png.matches(&[0xff, 0xd8, 0xff]));
        assert!(!ImageFormat::Jpeg.matches(b"\x89PNG\r\n\x1a\n"));
        assert!(!ImageFormat::Webp.matches(b"RIFF\0\0\0\0WAVE"));
        assert!(!ImageFormat::Avif.matches(b"\0\0\0\x14ftypheic\0\0\0\0heic"));
        assert!(!ImageFormat::Avif.matches(b"\0\0\0\x0cftypavif"[..10].as_ref()));
        assert!(!ImageFormat::Gif.matches(b""));
    }
}
