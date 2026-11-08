//! Arguments of `cuhlumn font`.

use std::path::PathBuf;

use clap::Subcommand;
use cuhlumn_api::site::link::AbsoluteUrl;

#[derive(Debug, Subcommand)]
pub enum FontCommand {
    /// Lists the fonts of the server library.
    List,
    /// Imports fonts from Google Fonts into the server library.
    ///
    /// The address is a specimen page or a css2 or css stylesheet address.
    /// A specimen page imports only the regular face.
    #[command(after_help = IMPORT_HELP)]
    Import {
        /// The Google Fonts address.
        url: AbsoluteUrl,
    },
    /// Uploads a TTF, OTF, or TTC font file of up to 20 MB to the server library.
    Upload {
        /// The font file.
        file: PathBuf,
    },
    /// Deletes a font from the server library.
    Delete {
        /// The font file name, as `font list` shows it.
        name: String,
        /// Confirms the deletion, which cannot be undone.
        #[arg(long)]
        yes: bool,
    },
}

const IMPORT_HELP: &str = "\
Examples:
  cuhlumn font import https://fonts.google.com/specimen/Lora
      imports only the regular face
  cuhlumn font import 'https://fonts.googleapis.com/css2?family=Lora:ital,wght@0,400;0,700;1,400'
      imports the regular, bold, and italic faces

The server downloads at most 20 files of up to 20 MB each.
Fonts are served publicly, so use fonts whose license allows web use, such as Google Fonts.";

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, Command};

    use super::FontCommand;

    #[test]
    fn import_parses_a_stylesheet_url() {
        let url = "https://fonts.googleapis.com/css2?family=Lora:ital,wght@0,400;0,700;1,400";
        let cli = Cli::try_parse_from(["cuhlumn", "font", "import", url]).unwrap();
        let Command::Font(FontCommand::Import { url: parsed }) = cli.command else {
            panic!("unexpected command: {:?}", cli.command);
        };
        assert_eq!(parsed.as_str(), url);
    }

    #[test]
    fn import_rejects_a_relative_url() {
        assert!(Cli::try_parse_from(["cuhlumn", "font", "import", "Lora"]).is_err());
    }
}
