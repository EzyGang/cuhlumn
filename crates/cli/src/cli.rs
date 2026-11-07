//! Command-line arguments.

pub mod announce;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use cuhlumn_api::{
    post::{Kind, Status, slug::Slug},
    site::{host::Host, link::AbsoluteUrl},
};

use crate::{assign::Assignment, cli::announce::AnnounceCommand};

/// Manages blogs on a cuhlumn server through its HTTP API.
///
/// Every command prints JSON on stdout, except `post get --markdown` and `skill`.
/// Errors go to stderr, and the exit code is 1.
#[derive(Debug, Parser)]
#[command(name = "cuhlumn", version, after_help = EXAMPLES)]
pub struct Cli {
    #[command(flatten)]
    pub connection: Connection,
    #[command(subcommand)]
    pub command: Command,
}

/// The server, the token, and the site that commands use.
#[derive(Debug, Args)]
#[command(next_help_heading = "Connection")]
pub struct Connection {
    /// Address of the cuhlumn server, such as https://blog.example.com.
    #[arg(long, env = "CUHLUMN_SERVER", global = true)]
    pub server: Option<AbsoluteUrl>,
    /// Admin token of the server.
    #[arg(long, env = "CUHLUMN_TOKEN", global = true, hide_env_values = true)]
    pub token: Option<String>,
    /// Canonical domain of the site to work on, such as blog.example.com.
    #[arg(long, env = "CUHLUMN_SITE", global = true)]
    pub site: Option<Host>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Checks that the server answers. Needs no token.
    Health,
    /// Lists, creates, changes, and deletes sites.
    #[command(subcommand)]
    Site(SiteCommand),
    /// Writes, publishes, and deletes posts and pages of the site.
    #[command(subcommand)]
    Post(PostCommand),
    /// Uploads, lists, and deletes images of the site.
    #[command(subcommand)]
    Media(MediaCommand),
    /// Shows, sets, and clears the announcement bar of the site.
    #[command(subcommand)]
    Announce(AnnounceCommand),
    /// Prints the agent skill that explains this CLI.
    Skill,
    /// Installs the agent skill into AI agents with the `skills` installer from npm.
    ///
    /// Runs `npx skills@latest add`, or `pnpm dlx skills@latest add`, with the skill of this binary.
    /// The installer asks for the agents and the scope unless you pass --agent, --global, and --yes.
    #[command(after_help = INSTALL_SKILL_HELP)]
    InstallSkill(InstallSkill),
}

/// Options of `install-skill`, passed to `skills add`.
#[derive(Debug, Args)]
pub struct InstallSkill {
    /// Installs for the current user instead of the current project.
    #[arg(short, long)]
    pub global: bool,
    /// Agents to install to, such as claude-code, codex, or cursor. Use '*' for all agents.
    #[arg(short, long = "agent", value_name = "AGENT", num_args = 1..)]
    pub agents: Vec<String>,
    /// Answers yes to every confirmation of the installer.
    #[arg(short, long)]
    pub yes: bool,
    /// Copies the files into each agent folder instead of linking them.
    #[arg(long)]
    pub copy: bool,
    /// The package runner that starts the installer.
    #[arg(long, value_enum, default_value_t = Runner::Auto)]
    pub runner: Runner,
    /// More options for `skills add`, after `--`.
    #[arg(last = true, value_name = "SKILLS_OPTIONS")]
    pub extra: Vec<String>,
}

/// The program that downloads and starts the `skills` installer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Runner {
    /// Uses npx, then pnpm when npx is not installed.
    Auto,
    Npx,
    Pnpm,
}

#[derive(Debug, Subcommand)]
pub enum SiteCommand {
    /// Lists every site on the server.
    List,
    /// Shows the site.
    Get {
        /// Prints the settings as an input file for `site apply`.
        #[arg(long)]
        input: bool,
    },
    /// Creates the site, or replaces all its settings, from a JSON or YAML file.
    ///
    /// Omitted fields take their default values. Use `-` to read stdin.
    Apply {
        /// The settings file.
        file: PathBuf,
    },
    /// Changes some settings of an existing site and keeps the others.
    #[command(after_help = SET_HELP)]
    Set {
        /// Assignments such as title=Notes or brand.credit:=false.
        #[arg(required = true)]
        assignments: Vec<Assignment>,
    },
    /// Deletes the site with its aliases, posts, pages, and images.
    Delete {
        /// Confirms the deletion, which cannot be undone.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum PostCommand {
    /// Lists the posts and pages of the site, drafts included, without their bodies.
    List {
        /// Shows only entries with this status.
        #[arg(long)]
        status: Option<Status>,
        /// Shows only entries of this kind.
        #[arg(long)]
        kind: Option<Kind>,
    },
    /// Shows one post or page.
    Get {
        slug: Slug,
        /// Prints a Markdown file with front matter for `post push`.
        #[arg(long)]
        markdown: bool,
    },
    /// Creates or replaces a post from a Markdown file with YAML front matter.
    ///
    /// The slug comes from the `slug` field, or from the file name.
    /// Omitted fields take their default values, and `status` defaults to draft.
    #[command(after_help = PUSH_HELP)]
    Push {
        /// The Markdown file.
        file: PathBuf,
        /// Publishes the post, whatever the front matter says.
        #[arg(long, conflicts_with = "draft")]
        publish: bool,
        /// Saves the post as a draft, also when it is published now.
        #[arg(long)]
        draft: bool,
    },
    /// Changes some fields of an existing post and keeps the others.
    #[command(after_help = SET_HELP)]
    Set {
        slug: Slug,
        /// Assignments such as title=News or tags:='["Ops"]'.
        #[arg(required = true)]
        assignments: Vec<Assignment>,
    },
    /// Makes a post or page visible to readers.
    Publish { slug: Slug },
    /// Hides a post or page from readers and keeps it as a draft.
    Unpublish { slug: Slug },
    /// Deletes a post or page.
    Delete {
        slug: Slug,
        /// Confirms the deletion, which cannot be undone.
        #[arg(long)]
        yes: bool,
    },
    /// Renders the body of a Markdown file as the blog shows it, without saving.
    Preview {
        /// The Markdown file, with or without front matter.
        file: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum MediaCommand {
    /// Uploads a PNG, JPEG, WebP, GIF, or AVIF image of up to 16 MiB.
    Upload {
        /// The image file. The extension sets the image type.
        file: PathBuf,
    },
    /// Lists the uploaded images of the site, newest first.
    List,
    /// Deletes an image. Posts that show it get a broken image.
    Delete {
        /// The image name, such as ba7816bf8f01cfea414140de5dae2223.png.
        name: String,
        /// Confirms the deletion, which cannot be undone.
        #[arg(long)]
        yes: bool,
    },
}

const EXAMPLES: &str = "\
Examples:
  export CUHLUMN_SERVER=https://blog.example.com CUHLUMN_TOKEN=... CUHLUMN_SITE=blog.example.com
  cuhlumn site apply site.json
  cuhlumn media upload diagram.png
  cuhlumn post push sqlite-backups.md --publish
  cuhlumn post get sqlite-backups --markdown > sqlite-backups.md
  cuhlumn announce set \"We moved to a new server.\" --until 2026-12-01T00:00:00Z
  cuhlumn skill";

const SET_HELP: &str = "\
Assignments:
  field=text     sets a string, for example title=Harbor Notes
  field:=json    sets any JSON value, for example brand.credit:=false, brand.logo:=null,
                 or aliases:='[\"www.blog.example.com\"]'
  a.b.c          sets a nested field, and a numeric part selects a list item";

const PUSH_HELP: &str = "\
File format:
  ---
  title: How we back up SQLite
  description: Our nightly routine for one database file.
  tags: [Databases]
  status: published
  ---

  We back up **one SQLite file** every night.

Fields: slug, title, description, tags, kind (post or page), status (draft or published),
cover, og_image, published_at (unix seconds, 2026-05-01, or 2026-05-01T09:30:00Z).
A published post stays published only when the file says status: published or you pass --publish.
The command refuses to turn a published post into a draft unless you pass --draft.";

const INSTALL_SKILL_HELP: &str = "\
Examples:
  cuhlumn install-skill                         asks for the agents and the scope
  cuhlumn install-skill -g -a claude-code -y    installs for Claude Code for this user
  cuhlumn install-skill -a '*' -y               installs for every agent in this project

The installer needs Node.js (npx) or pnpm. Without them, save the output of `cuhlumn skill`
as SKILL.md in a folder named cuhlumn in the skills folder of your agent.
Run the command again after you upgrade cuhlumn.";
