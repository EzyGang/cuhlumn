//! `cuhlumn media` commands.

use std::{fs, path::Path};

use anyhow::{Context as _, bail};
use serde::Serialize;

use crate::{
    cli::MediaCommand,
    commands::{Context, confirm, print_json},
};

/// The result of an upload, with the root-relative path to use in posts and settings.
#[derive(Debug, Serialize)]
struct Uploaded {
    name: String,
    url: String,
    path: String,
}

pub(super) fn run(context: &Context, command: MediaCommand) -> anyhow::Result<()> {
    let client = &context.client;
    match command {
        MediaCommand::Upload { file } => {
            let content_type = content_type(&file)?;
            let bytes =
                fs::read(&file).with_context(|| format!("cannot read {}", file.display()))?;
            let saved = client.upload(context.site()?, content_type, bytes)?;
            print_json(&Uploaded {
                path: format!("/media/{}", saved.name),
                name: saved.name,
                url: saved.url,
            })
        }
        MediaCommand::List => print_json(&client.media(context.site()?)?),
        MediaCommand::Delete { name, yes } => {
            confirm(yes, &format!("the image {name}"))?;
            Ok(client.delete_media(context.site()?, &name)?)
        }
    }
}

/// Returns the image type that the file extension names.
fn content_type(file: &Path) -> anyhow::Result<&'static str> {
    let extension = file
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("png") => Ok("image/png"),
        Some("jpg" | "jpeg") => Ok("image/jpeg"),
        Some("webp") => Ok("image/webp"),
        Some("gif") => Ok("image/gif"),
        Some("avif") => Ok("image/avif"),
        _ => bail!(
            "{}: the server accepts only .png, .jpg, .jpeg, .webp, .gif, and .avif images",
            file.display()
        ),
    }
}
