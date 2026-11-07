//! `cuhlumn announce` commands.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use cuhlumn_api::site::{Site, SiteInput, announcement::Announcement};
use serde::Serialize;
use serde_json::json;

use crate::{
    cli::announce::{AnnounceCommand, AnnounceSet},
    commands::{Context, print_json},
};

/// The output of every `announce` command.
#[derive(Serialize)]
struct Shown {
    /// `off`, `scheduled`, `live`, or `ended`.
    status: &'static str,
    announcement: Option<Announcement>,
}

pub(super) fn run(context: &Context, command: AnnounceCommand) -> anyhow::Result<()> {
    let client = &context.client;
    let host = context.site()?;
    let site = match command {
        AnnounceCommand::Show => client.site(host)?,
        AnnounceCommand::Set(options) => {
            let announcement = announcement(options)?;
            client.put_site(
                host,
                &with_announcement(client.site(host)?, Some(announcement)),
            )?
        }
        AnnounceCommand::Clear => {
            client.put_site(host, &with_announcement(client.site(host)?, None))?
        }
    };
    print_json(&shown(site)?)
}

/// Builds the announcement through its validation rules, so errors name the field.
fn announcement(options: AnnounceSet) -> anyhow::Result<Announcement> {
    let value = json!({
        "text": options.text,
        "link": options.link,
        "link_label": options.label,
        "tone": options.tone,
        "starts_at": options.from,
        "ends_at": options.until,
        "dismissible": !options.no_dismiss,
    });
    serde_path_to_error::deserialize(value).context("the announcement is not valid")
}

fn with_announcement(site: Site, announcement: Option<Announcement>) -> SiteInput {
    SiteInput {
        announcement,
        ..SiteInput::from(site)
    }
}

fn shown(site: Site) -> anyhow::Result<Shown> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("the system clock is before 1970")?;
    let now = i64::try_from(now.as_secs()).context("the system clock is out of range")?;
    let status = site
        .announcement
        .as_ref()
        .map_or("off", |announcement| announcement.status(now).as_str());
    Ok(Shown {
        status,
        announcement: site.announcement,
    })
}
