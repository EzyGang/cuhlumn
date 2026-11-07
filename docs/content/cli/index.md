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

When the server uses an admin port (`CUHLUMN_ADMIN_ADDR`), `CUHLUMN_SERVER` must be the admin address, for example `http://100.92.3.4:8081`.
The public address then answers only `cuhlumn health`, and the other commands get `404`.

`cuhlumn health` needs only the server address.
`cuhlumn site list` needs the server address and the token.
Every other site, post, media, and announce command also needs the site.

A new server without a token is in setup mode.
The admin sets the token on the **Setup** page of the console, or sets `CUHLUMN_TOKEN` on the server.
Until then, admin commands fail with the `needs_setup` error.
The client uses the token that the admin set.

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
| `cuhlumn announce show` | The announcement bar and its status |
| `cuhlumn announce set <text> [options]` | Replaces the announcement bar |
| `cuhlumn announce clear` | Removes the announcement bar |
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
| `show_cta` | Site default | `true` or `false`. Without it, the post follows the site `show_cta` |
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

### Appearance and IndexNow fields

```sh
cuhlumn site set indexnow:=false
cuhlumn site set brand.palette=sepia brand.home_layout=cards
cuhlumn site set brand.code_theme=ocean brand.card_style=light
cuhlumn site set 'brand.social:=[{"network": "github", "url": "https://github.com/maralind"}]'
```

| Field | Default | Values |
| --- | --- | --- |
| `indexnow` | `true` | `true` sends IndexNow pings when an entry is published |
| `brand.palette` | `paper` | `paper`, `slate`, `sepia`, `mono`, `contrast` |
| `brand.theme` | `system` | `system`, `light`, `dark` |
| `brand.theme_toggle` | `true` | `true`, `false` |
| `brand.width` | `normal` | `narrow`, `normal`, `wide` |
| `brand.home_layout` | `list` | `list`, `compact`, `cards` |
| `brand.code_theme` | `github` | `github`, `solarized`, `ocean`, `none` |
| `brand.card_style` | `dark` | `dark`, `light`, `accent` |
| `brand.card_meta` | `true` | `true`, `false` |
| `brand.social` | `[]` | Up to 12 `{"network", "url"}` objects, without repetitions |
| `brand.social_in_header` | `false` | `true`, `false` |

The social networks are `github`, `gitlab`, `codeberg`, `mastodon`, `bluesky`, `x`, `threads`, `linkedin`, `youtube`, `instagram`, `discord`, `telegram`, and `email`.
Each `url` is an absolute `https://` address, at most 500 characters.
For `email`, the `url` is a `mailto:` address with one recipient.

## Announcement bar

`cuhlumn announce` shows, replaces, or removes the announcement bar of the site.

```sh
cuhlumn announce set "Maintenance on Sunday." --tone warning \
  --link https://harbor.example.com/maintenance --label Details \
  --from 2026-11-10T09:00:00Z --until 2026-11-15T18:00:00Z
cuhlumn announce show
cuhlumn announce clear
```

| Option | Default | Meaning |
| --- | --- | --- |
| `<text>` | Required | Plain text, 1 to 160 characters |
| `--link URL` | None | The address that the link opens |
| `--label TEXT` | `Learn more` | The link text, at most 40 characters. It needs `--link` |
| `--tone T` | `info` | `info`, `accent`, or `warning` |
| `--from TIME` | Now | The start time |
| `--until TIME` | Until you clear it | The end time, later than the start time |
| `--no-dismiss` | Off | Removes the close button |

TIME is an RFC 3339 time, such as `2026-11-10T09:00:00Z`, or unix seconds, such as `1794301200`.
`announce set` replaces the whole announcement.
Omitted options take their defaults, also when the old announcement had other values.

Every `announce` command prints the status and the announcement:

```json
{"status": "scheduled", "announcement": {"text": "Maintenance on Sunday.", "link": "https://harbor.example.com/maintenance", "link_label": "Details", "tone": "warning", "starts_at": 1794301200, "ends_at": 1794765600, "dismissible": true}}
```

| Status | Meaning |
| --- | --- |
| `off` | The site has no announcement, and `announcement` is `null` |
| `scheduled` | The start time is in the future |
| `live` | Readers see the bar now |
| `ended` | The end time is in the past |

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
| `1` | The command failed. Stderr has one `error:` line |
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
