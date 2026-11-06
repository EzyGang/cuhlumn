//! Runs the parsed commands against the server.

mod media;
mod post;
mod site;
mod skill;

use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};

use anyhow::{Context as _, bail};
use cuhlumn_api::site::host::Host;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
    assign::Assignment,
    cli::{Cli, Command, Connection},
    client::Client,
};

/// Runs one command and prints its result.
///
/// # Errors
///
/// Returns an error when the settings are missing, a file is invalid, or the server rejects the request.
pub fn run(cli: Cli) -> anyhow::Result<()> {
    let Cli {
        connection,
        command,
    } = cli;
    match command {
        Command::Skill => write_stdout(skill::SKILL),
        Command::InstallSkill(options) => skill::install(options),
        Command::Health => print_json(&connect(&connection, false)?.health()?),
        Command::Site(command) => site::run(&Context::admin(connection)?, command),
        Command::Post(command) => post::run(&Context::admin(connection)?, command),
        Command::Media(command) => media::run(&Context::admin(connection)?, command),
    }
}

/// The client and the site for commands that need the admin token.
struct Context {
    client: Client,
    site: Option<Host>,
}

impl Context {
    fn admin(connection: Connection) -> anyhow::Result<Self> {
        let client = connect(&connection, true)?;
        Ok(Self {
            client,
            site: connection.site,
        })
    }

    /// Returns the site that the command works on.
    fn site(&self) -> anyhow::Result<&Host> {
        self.site
            .as_ref()
            .context("this command needs a site: set CUHLUMN_SITE or pass --site")
    }
}

fn connect(connection: &Connection, admin: bool) -> anyhow::Result<Client> {
    let server = connection
        .server
        .as_ref()
        .context("set CUHLUMN_SERVER or pass --server with the server address")?;
    if admin && connection.token.is_none() {
        bail!("this command needs the admin token: set CUHLUMN_TOKEN or pass --token");
    }
    Ok(Client::new(server, connection.token.clone())?)
}

/// Prints a value as indented JSON with a final line break.
fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    write_stdout(&text)
}

fn write_stdout(text: &str) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(text.as_bytes())?;
    Ok(stdout.flush()?)
}

/// Reads a text file, or stdin when the path is `-`.
fn read_text(path: &Path) -> anyhow::Result<String> {
    if path == Path::new("-") {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .context("cannot read stdin")?;
        return Ok(text);
    }
    fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))
}

/// Applies assignments to the JSON form of `current` and checks the result as `T`.
fn assigned<C, T>(current: &C, assignments: &[Assignment]) -> anyhow::Result<T>
where
    C: Serialize,
    T: DeserializeOwned,
{
    let mut value: Value = serde_json::to_value(current)?;
    for assignment in assignments {
        assignment.apply(&mut value)?;
    }
    Ok(serde_path_to_error::deserialize(value)?)
}

/// Fails unless the user confirmed a deletion with `--yes`.
fn confirm(yes: bool, what: &str) -> anyhow::Result<()> {
    if yes {
        return Ok(());
    }
    bail!("deleting {what} cannot be undone: add --yes to confirm")
}
