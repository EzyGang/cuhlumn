//! `cuhlumn font` commands for the server-wide font library.

use std::fs;

use anyhow::Context as _;

use crate::{
    cli::font::FontCommand,
    commands::{Context, confirm, print_json},
};

pub(super) fn run(context: &Context, command: FontCommand) -> anyhow::Result<()> {
    let client = &context.client;
    match command {
        FontCommand::List => print_json(&client.fonts()?),
        FontCommand::Import { url } => print_json(&client.import_google_font(url)?),
        FontCommand::Upload { file } => {
            let bytes =
                fs::read(&file).with_context(|| format!("cannot read {}", file.display()))?;
            print_json(&client.upload_font(bytes)?)
        }
        FontCommand::Delete { name, yes } => {
            confirm(yes, &format!("the font {name}"))?;
            Ok(client.delete_font(&name)?)
        }
    }
}
