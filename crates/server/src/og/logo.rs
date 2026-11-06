//! The site logo as image bytes for share cards.

use std::sync::Arc;

use cuhlumn_api::site::{Site, host::Host};
use resvg::usvg::ImageKind;
use tokio::io::AsyncReadExt;

use crate::{
    media::{MediaStore, format::ImageFormat, name::MediaName},
    og::OgError,
};

/// Reads the uploaded logo of a site.
///
/// Returns `None` when the logo is not set, is not an upload of this site, is not PNG, JPEG, or GIF,
/// or does not exist on disk.
///
/// # Errors
///
/// Returns [`OgError`] when the logo file exists but cannot be read.
pub async fn load(media: &MediaStore, site: &Site) -> Result<Option<ImageKind>, OgError> {
    let Some(name) = site
        .brand
        .logo
        .as_ref()
        .and_then(|logo| upload_name(logo.as_str(), &site.host))
    else {
        return Ok(None);
    };
    let kind: fn(Arc<Vec<u8>>) -> ImageKind = match name.format() {
        ImageFormat::Png => ImageKind::PNG,
        ImageFormat::Jpeg => ImageKind::JPEG,
        ImageFormat::Gif => ImageKind::GIF,
        ImageFormat::Webp | ImageFormat::Avif => return Ok(None),
    };
    let Some(mut file) = media.open(&site.host, &name).await? else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await.map_err(OgError::Logo)?;
    Ok(Some(kind(Arc::new(bytes))))
}

/// Returns the media name of `/media/{name}` or of an absolute address of it on `host`.
fn upload_name(url: &str, host: &Host) -> Option<MediaName> {
    let path = ["https://", "http://"]
        .iter()
        .find_map(|scheme| url.strip_prefix(scheme)?.strip_prefix(host.as_str()))
        .unwrap_or(url);
    MediaName::new(path.strip_prefix("/media/")?).ok()
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::host::Host;

    use crate::og::logo::upload_name;

    #[test]
    fn only_uploads_of_the_site_count() {
        let host = Host::new("blog.example.com").unwrap();
        let name = "0123456789abcdef0123456789abcdef.png";
        for url in [
            format!("/media/{name}"),
            format!("https://blog.example.com/media/{name}"),
            format!("http://blog.example.com/media/{name}"),
        ] {
            assert_eq!(upload_name(&url, &host).unwrap().as_str(), name, "{url}");
        }
        for url in [
            format!("https://other.example.com/media/{name}"),
            format!("https://blog.example.com.evil.org/media/{name}"),
            "/media/../cuhlumn.db".to_owned(),
            "/logo.png".to_owned(),
        ] {
            assert!(upload_name(&url, &host).is_none(), "{url}");
        }
    }
}
