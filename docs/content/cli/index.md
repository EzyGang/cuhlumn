# Command-line client

The `cuhlumn` command-line client calls the HTTP API.
Writers, scripts, and AI agents use it to publish Markdown files and to manage sites and images.
Every command prints JSON, so scripts and agents can read the result.

## Installation

Build the client from a checkout of the repository:

```sh
cargo install --path crates/cli
```

The command installs the `cuhlumn` binary in `~/.cargo/bin`.

## Settings

| Variable | Flag | Meaning |
| --- | --- | --- |
| `CUHLUMN_SERVER` | `--server` | Any address of the server, such as `https://blog.example.com` |
| `CUHLUMN_TOKEN` | `--token` | The admin token |
| `CUHLUMN_SITE` | `--site` | The canonical domain of the site that commands work on |

`cuhlumn health` needs only the server address.
`cuhlumn site list` needs the server address and the token.
Every other site, post, and media command also needs the site.

## Commands

| Command | Result |
| --- | --- |
| `cuhlumn health` | Server status and version |
| `cuhlumn site list` | All sites |
| `cuhlumn site get [--input]` | The site, or its settings as an input file |
| `cuhlumn site apply <file>` | Creates the site, or replaces all its settings, from JSON or YAML |
| `cuhlumn site set <field=value>...` | Changes some settings and keeps the others |
| `cuhlumn site delete --yes` | Deletes the site with its posts and images |
| `cuhlumn post list [--status S] [--kind K]` | Posts and pages without their bodies |
| `cuhlumn post get <slug> [--markdown]` | One entry as JSON, or as a Markdown file |
| `cuhlumn post push <file> [--publish \| --draft]` | Creates or replaces an entry from a Markdown file |
| `cuhlumn post set <slug> <field=value>...` | Changes some fields and keeps the others |
| `cuhlumn post publish <slug>` | Makes the entry visible |
| `cuhlumn post unpublish <slug>` | Hides the entry and keeps it as a draft |
| `cuhlumn post preview <file>` | Renders the body without saving |
| `cuhlumn post delete <slug> --yes` | Deletes the entry |
| `cuhlumn media upload <file>` | Uploads an image and prints its name, address, and `/media/` path |
| `cuhlumn media list` | Uploaded images |
| `cuhlumn media delete <name> --yes` | Deletes an image |
| `cuhlumn skill` | Prints the agent skill |
| `cuhlumn install-skill` | Installs the agent skill into AI agents with the `skills` installer |

`cuhlumn --help` and `cuhlumn <command> --help` show every option.

## Post files

A post file is Markdown with YAML front matter:

```markdown
---
title: How we back up SQLite
description: Our nightly routine for one database file.
tags: [Databases, Ops]
status: published
---

We back up **one SQLite file** every night.
```

| Field | Default | Meaning |
| --- | --- | --- |
| `slug` | The file name without `.md` | The address of the entry |
| `title` | Required | The title |
| `description` | Empty | The summary for lists, search results, and share images |
| `tags` | None | Up to 20 tags |
| `kind` | `post` | `post` or `page` |
| `status` | `draft` | `draft` or `published` |
| `cover` | None | The cover image address |
| `og_image` | None | The share image address |
| `published_at` | First publication | Unix seconds, `2026-05-01`, or an RFC 3339 time |

The client checks the file before it sends it.
Unknown fields, a missing title, an empty body, or an invalid value stop the command with a message that names the field.

### Replacement and drafts

`post push` replaces the complete entry.
To edit an entry, start from the stored version:

```sh
cuhlumn post get sqlite-backups --markdown > sqlite-backups.md
# edit sqlite-backups.md
cuhlumn post push sqlite-backups.md
```

`post get --markdown` writes every field, so the file keeps the status and the publication time.

`post push` refuses to turn a published entry into a draft by accident.
The file must say `status: published`, or you pass `--publish` to keep the entry public or `--draft` to hide it.

## Field assignments

`site set` and `post set` read the stored object, change the named fields, and save the result.

| Syntax | Effect | Example |
| --- | --- | --- |
| `field=text` | Sets a string | `title=Harbor Notes` |
| `field:=json` | Sets any JSON value | `brand.credit:=false`, `cover:=null` |
| `a.b.c=...` | Sets a nested field | `brand.accent=#0f766e` |
| `list.1=...` | Replaces an existing list item | `aliases.0=www.blog.example.com` |

The client checks the result before it sends it.

## Images

`media upload` sends the file with the type from its extension: `.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, or `.avif`.

```json
{
  "name": "ba7816bf8f01cfea414140de5dae2223.png",
  "url": "https://blog.example.com/media/ba7816bf8f01cfea414140de5dae2223.png",
  "path": "/media/ba7816bf8f01cfea414140de5dae2223.png"
}
```

Use `path` in posts and settings, because it works on every domain and port.

## Output and exit codes

| Exit code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | The command failed; stderr has one `error:` line |
| `2` | The command line is invalid |

Server errors start with the [error code](../reference/errors.md), for example `error: conflict: ...`.
Delete commands print nothing on success.
`post get --markdown`, `skill`, and `install-skill` print text instead of JSON.

## Agent skill

The file `skills/cuhlumn/SKILL.md` explains the client to AI agents.
The binary contains the same file, so the installed skill always matches the installed client.

### Install the skill

`cuhlumn install-skill` installs the skill with the [`skills`](https://www.npmjs.com/package/skills) installer, the same installer that other agent skills use.

```sh
cuhlumn install-skill                         # asks for the agents and the scope
cuhlumn install-skill -g -a claude-code -y    # Claude Code, for this user
cuhlumn install-skill -a '*' -y               # every agent, in this project
```

The command does these steps:

1. It writes the skill of the binary to `<user cache>/cuhlumn/skill/skills/cuhlumn/SKILL.md`.
2. It runs `npx --yes skills@latest add <that folder>`, or `pnpm dlx skills@latest add <that folder>` when npx is not installed.
3. The installer copies or links the skill into the skills folder of each agent that you select.

| Option | Passed to `skills add` as | Effect |
| --- | --- | --- |
| `-g`, `--global` | `--global` | Installs for the current user instead of the current project |
| `-a`, `--agent <AGENT>...` | `--agent` | Selects agents, such as `claude-code`, `codex`, or `cursor`, or `'*'` for all agents |
| `-y`, `--yes` | `--yes` | Skips the confirmations |
| `--copy` | `--copy` | Copies files instead of linking them |
| `--runner auto\|npx\|pnpm` | Not passed | Selects the package runner, and `auto` tries npx, then pnpm |
| `-- <options>` | As written | Passes other installer options, such as `-- --json` |

The user cache is `%LOCALAPPDATA%` on Windows, `~/Library/Caches` on macOS, and `$XDG_CACHE_HOME` or `~/.cache` on Linux.
A project install records that folder in `skills-lock.json`.
Run `cuhlumn install-skill` again after you upgrade the client.

### Install the skill without Node.js

The installer needs Node.js with npx, or pnpm.
Without them, save the skill in the skills folder of your agent:

```sh
mkdir -p ~/.agents/skills/cuhlumn
cuhlumn skill > ~/.agents/skills/cuhlumn/SKILL.md
```
