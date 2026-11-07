//! The font library of share images.

use serde::{Deserialize, Serialize};

use crate::site::link::AbsoluteUrl;

text_enum! {
    /// The origin of a library font.
    pub enum FontSource / FontSourceError ("font source") {
        /// An uploaded file.
        #[default]
        Upload = "upload",
        /// A file imported from Google Fonts.
        Google = "google",
    }
}

/// One font file in `GET /api/fonts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FontItem {
    /// The file name in the font folder.
    pub name: String,
    /// The family names in the file.
    pub families: Vec<String>,
    pub source: FontSource,
    /// The file size in bytes.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub size: u64,
    /// The upload time, in unix seconds.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub created_at: i64,
}

/// The body of `POST /api/fonts/google`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct GoogleFontInput {
    /// A Google Fonts specimen or stylesheet address.
    pub url: AbsoluteUrl,
}
