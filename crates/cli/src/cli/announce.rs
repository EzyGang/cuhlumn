//! Arguments of `cuhlumn announce`.

use chrono::DateTime;
use clap::{Args, Subcommand};
use cuhlumn_api::site::{announcement::Tone, link::LinkUrl};

#[derive(Debug, Subcommand)]
pub enum AnnounceCommand {
    /// Shows the announcement of the site and its status: off, scheduled, live, or ended.
    Show,
    /// Replaces the announcement of the site.
    ///
    /// Omitted options take their default values, also when the old announcement had other values.
    #[command(after_help = SET_HELP)]
    Set(AnnounceSet),
    /// Removes the announcement of the site.
    Clear,
}

/// Options of `announce set`.
#[derive(Debug, Args)]
pub struct AnnounceSet {
    /// Plain text of the bar, 1 to 160 characters.
    pub text: String,
    /// Address that the link of the bar opens.
    #[arg(long)]
    pub link: Option<LinkUrl>,
    /// Text of the link, at most 40 characters. The default is "Learn more".
    #[arg(long, requires = "link")]
    pub label: Option<String>,
    /// Colour of the bar: info, accent, or warning.
    #[arg(long, default_value_t = Tone::Info)]
    pub tone: Tone,
    /// Time when the bar starts. Without it, the bar starts now.
    #[arg(long, value_name = "TIME", value_parser = parse_time)]
    pub from: Option<i64>,
    /// Time when the bar ends. Without it, the bar stays until you clear it.
    #[arg(long, value_name = "TIME", value_parser = parse_time)]
    pub until: Option<i64>,
    /// Removes the close button, so readers cannot hide the bar.
    #[arg(long)]
    pub no_dismiss: bool,
}

const SET_HELP: &str = "\
TIME is an RFC 3339 time, such as 2026-11-10T09:00:00Z or 2026-11-10T11:00:00+02:00,
or unix seconds, such as 1794301200.

Examples:
  cuhlumn announce set \"We moved to a new server.\" --tone accent
  cuhlumn announce set \"Maintenance on Sunday.\" --tone warning \\
    --link https://harbor.example.com/maintenance --label Details \\
    --from 2026-11-10T09:00:00Z --until 2026-11-15T18:00:00Z
  cuhlumn announce show
  cuhlumn announce clear";

/// Parses an RFC 3339 time or unix seconds as unix seconds.
fn parse_time(text: &str) -> Result<i64, String> {
    if let Ok(seconds) = text.parse::<i64>() {
        return Ok(seconds);
    }
    DateTime::parse_from_rfc3339(text)
        .map(|time| time.timestamp())
        .map_err(|_| {
            "use an RFC 3339 time, such as 2026-11-10T09:00:00Z, or unix seconds".to_owned()
        })
}

#[cfg(test)]
mod tests {
    use super::parse_time;

    #[test]
    fn times_accept_rfc_3339_and_unix_seconds() {
        assert_eq!(parse_time("1794301200"), Ok(1_794_301_200));
        assert_eq!(parse_time("2026-11-10T09:00:00Z"), Ok(1_794_301_200));
        assert_eq!(parse_time("2026-11-10T11:00:00+02:00"), Ok(1_794_301_200));
        assert!(parse_time("2026-11-10").is_err());
        assert!(parse_time("tomorrow").is_err());
    }
}
