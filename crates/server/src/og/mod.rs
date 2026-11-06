//! Share images: a 1200×630 PNG card for every post and site, drawn from SVG with the server fonts.

pub mod card;
mod fit;
mod fonts;
mod logo;
mod palette;

use std::{error::Error, io, path::PathBuf, sync::Arc};

use cuhlumn_api::site::Site;
use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{self, ImageHrefResolver, ImageKind},
};

use crate::{
    media::{MediaError, MediaStore},
    og::card::{Card, LOGO_HREF},
};

/// The text of a share card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardText {
    pub title: String,
    pub subtitle: String,
    /// The day and reading time of a post, or nothing.
    pub meta: String,
}

/// A failure while a share card renders.
#[derive(Debug, thiserror::Error)]
pub enum OgError {
    /// The generated SVG cannot be parsed.
    #[error("the card SVG is not valid: {0}")]
    Svg(#[from] usvg::Error),
    /// The SVG has no area to draw on.
    #[error("the card has an empty size")]
    EmptySize,
    /// The pixels cannot be encoded as PNG.
    #[error("PNG encoding failed: {0}")]
    Encode(Box<dyn Error + Send + Sync>),
    /// The logo file cannot be opened.
    #[error(transparent)]
    Media(#[from] MediaError),
    /// The logo file cannot be read.
    #[error("the logo file cannot be read: {0}")]
    Logo(io::Error),
}

/// A share card with its logo bytes, ready to draw.
pub struct PreparedCard {
    card: Card,
    logo: Option<ImageKind>,
    fonts: Option<PathBuf>,
}

/// Reads the logo of `site` and lays out its share card.
///
/// `fonts` is the extra font folder from `CUHLUMN_FONTS`.
///
/// # Errors
///
/// Returns [`OgError`] when the logo file exists but cannot be read.
pub async fn prepare(
    media: &MediaStore,
    fonts: Option<PathBuf>,
    site: &Site,
    text: CardText,
) -> Result<PreparedCard, OgError> {
    let logo = logo::load(media, site).await?;
    let card = Card {
        site: site.title.clone(),
        title: text.title,
        subtitle: text.subtitle,
        host: site.host.to_string(),
        meta: text.meta,
        accent: site.brand.accent.clone(),
        logo: logo.is_some(),
    };
    Ok(PreparedCard { card, logo, fonts })
}

impl PreparedCard {
    /// Draws the card as PNG.
    ///
    /// The work is CPU-bound and the first call loads the fonts, so call it on the blocking thread pool.
    ///
    /// # Errors
    ///
    /// Returns [`OgError`] when the SVG cannot be parsed, drawn, or encoded.
    pub fn draw(self) -> Result<Vec<u8>, OgError> {
        let Self { card, logo, fonts } = self;
        let options = usvg::Options {
            fontdb: fonts::database(fonts.as_deref()),
            image_href_resolver: ImageHrefResolver {
                resolve_data: Box::new(|_, _, _| None),
                resolve_string: Box::new(move |href, _| {
                    logo.as_ref().filter(|_| href == LOGO_HREF).map(clone_kind)
                }),
            },
            ..usvg::Options::default()
        };
        let tree = usvg::Tree::from_str(&card.svg(), &options)?;
        let size = tree.size().to_int_size();
        let mut pixmap = Pixmap::new(size.width(), size.height()).ok_or(OgError::EmptySize)?;
        resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
        pixmap
            .encode_png()
            .map_err(|err| OgError::Encode(Box::new(err)))
    }
}

fn clone_kind(kind: &ImageKind) -> ImageKind {
    match kind {
        ImageKind::JPEG(bytes) => ImageKind::JPEG(Arc::clone(bytes)),
        ImageKind::PNG(bytes) => ImageKind::PNG(Arc::clone(bytes)),
        ImageKind::GIF(bytes) => ImageKind::GIF(Arc::clone(bytes)),
        ImageKind::WEBP(bytes) => ImageKind::WEBP(Arc::clone(bytes)),
        ImageKind::SVG(tree) => ImageKind::SVG(tree.clone()),
    }
}
