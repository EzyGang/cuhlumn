//! `cuhlumn site` commands.

use anyhow::Context as _;
use cuhlumn_api::site::SiteInput;
use serde_json::Value;

use crate::{
    cli::SiteCommand,
    commands::{Context, assigned, confirm, print_json, read_text},
};

pub(super) fn run(context: &Context, command: SiteCommand) -> anyhow::Result<()> {
    let client = &context.client;
    match command {
        SiteCommand::List => print_json(&client.sites()?),
        SiteCommand::Get { input } => {
            let site = client.site(context.site()?)?;
            if input {
                return print_json(&SiteInput::from(site));
            }
            print_json(&site)
        }
        SiteCommand::Apply { file } => {
            let text = read_text(&file)?;
            let value: Value = serde_saphyr::from_str(&text)
                .with_context(|| format!("{} is not valid JSON or YAML", file.display()))?;
            let input: SiteInput = serde_path_to_error::deserialize(value)
                .with_context(|| format!("{} is not a valid site", file.display()))?;
            print_json(&client.put_site(context.site()?, &input)?)
        }
        SiteCommand::Set { assignments } => {
            let host = context.site()?;
            let current = SiteInput::from(client.site(host)?);
            let input: SiteInput = assigned(&current, &assignments)?;
            print_json(&client.put_site(host, &input)?)
        }
        SiteCommand::Delete { yes } => {
            let host = context.site()?;
            confirm(yes, &format!("the site {host}"))?;
            Ok(client.delete_site(host)?)
        }
    }
}
