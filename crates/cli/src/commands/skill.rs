//! `cuhlumn skill` and `cuhlumn install-skill`.

use std::{
    ffi::OsString,
    fs, io,
    path::PathBuf,
    process::{Command, ExitStatus},
};

use anyhow::{Context as _, bail};

use crate::cli::{InstallSkill, Runner};

/// The agent skill that explains this CLI.
pub(super) const SKILL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../skills/cuhlumn/SKILL.md"
));

/// Installs the skill of this binary with the `skills` installer from npm.
///
/// The skill goes to a fixed folder in the user cache first, because the installer reads a local folder
/// and records its path in `skills-lock.json`.
pub(super) fn install(options: InstallSkill) -> anyhow::Result<()> {
    let source = write_source()?;
    let args = skills_add_args(source, &options);
    let runners: &[Runner] = match options.runner {
        Runner::Auto => &[Runner::Npx, Runner::Pnpm],
        Runner::Npx => &[Runner::Npx],
        Runner::Pnpm => &[Runner::Pnpm],
    };
    for runner in runners {
        match run(*runner, &args) {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => bail!("the skills installer failed ({status})"),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("cannot start {}", runner_program(*runner)));
            }
        }
    }
    bail!(
        "install-skill needs Node.js with npx, or pnpm: install one of them, \
         or save the output of `cuhlumn skill` as cuhlumn/SKILL.md in the skills folder of your agent"
    )
}

/// Writes the skill to `<cache>/cuhlumn/skill/skills/cuhlumn/SKILL.md` and returns the source folder.
fn write_source() -> anyhow::Result<PathBuf> {
    let source = dirs::cache_dir()
        .context("cannot find the cache folder of this user")?
        .join("cuhlumn")
        .join("skill");
    let folder = source.join("skills").join("cuhlumn");
    fs::create_dir_all(&folder).with_context(|| format!("cannot create {}", folder.display()))?;
    let file = folder.join("SKILL.md");
    fs::write(&file, SKILL).with_context(|| format!("cannot write {}", file.display()))?;
    Ok(source)
}

fn skills_add_args(source: PathBuf, options: &InstallSkill) -> Vec<OsString> {
    let mut args: Vec<OsString> = vec!["skills@latest".into(), "add".into(), source.into()];
    let flags = [
        (options.global, "--global"),
        (options.yes, "--yes"),
        (options.copy, "--copy"),
    ];
    args.extend(
        flags
            .into_iter()
            .filter(|(set, _)| *set)
            .map(|(_, flag)| flag.into()),
    );
    if !options.agents.is_empty() {
        args.push("--agent".into());
        args.extend(options.agents.iter().map(OsString::from));
    }
    args.extend(options.extra.iter().map(OsString::from));
    args
}

/// Starts the installer with the terminal of this process, so its prompts work.
fn run(runner: Runner, args: &[OsString]) -> io::Result<ExitStatus> {
    let mut command = Command::new(runner_program(runner));
    match runner {
        Runner::Npx => command.arg("--yes"),
        Runner::Pnpm | Runner::Auto => command.arg("dlx"),
    };
    command.args(args).status()
}

/// Returns the program name. On Windows, npx and pnpm are `.cmd` scripts.
fn runner_program(runner: Runner) -> &'static str {
    match (runner, cfg!(windows)) {
        (Runner::Npx, true) => "npx.cmd",
        (Runner::Npx, false) => "npx",
        (Runner::Pnpm | Runner::Auto, true) => "pnpm.cmd",
        (Runner::Pnpm | Runner::Auto, false) => "pnpm",
    }
}
