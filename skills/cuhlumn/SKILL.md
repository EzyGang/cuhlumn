---
name: cuhlumn
description: Use to create, change, publish, or delete blogs, posts, pages, and images on a Cuhlumn server with the cuhlumn CLI.
---

# cuhlumn

The `cuhlumn` CLI manages blogs on a Cuhlumn server through the server HTTP API.
One server hosts many blogs, and each blog (a "site") has its own domain.
Posts are Markdown files with YAML front matter.

## Setup

Set these environment variables before you run a command:

| Variable | Value |
| --- | --- |
| `CUHLUMN_SERVER` | Any address of the server, such as `https://blog.example.com` |
| `CUHLUMN_TOKEN` | The admin token |
| `CUHLUMN_SITE` | The canonical domain of the blog, such as `blog.example.com` |

The flags `--server`, `--token`, and `--site` override the variables.
When the server uses an admin port (`CUHLUMN_ADMIN_ADDR`), `CUHLUMN_SERVER` must be the admin address, for example `http://100.92.3.4:8081`.
The public address then answers only `cuhlumn health`.
`cuhlumn health` checks the connection without a token.
Never print the token or write it into a file in a repository.

A new server without a token is in setup mode.
The admin sets the token on the **Setup** page of the console, or sets `CUHLUMN_TOKEN` on the server.
Until then, admin commands fail with `error: needs_setup: ...`.
Use the token that the admin set.

## Output and errors

- Every command prints JSON on stdout, except `post get --markdown`, `skill`, and `install-skill`.
- Delete commands print nothing.
- On failure, the command prints one `error: ...` line on stderr and exits with code 1.
- Server errors start with a code, such as `error: not_found: ...` or `error: validation: title: ...`.
- The CLI checks fields before it sends a request, and the message names the field.
- Usage errors exit with code 2.

## Publish a post

1. Write a Markdown file with front matter:

   ```markdown
   ---
   title: How we back up SQLite
   description: Our nightly routine for one database file.
   tags: [Databases, Ops]
   status: published
   ---

   We back up **one SQLite file** every night.

   ## Why one file

   ...
   ```

2. Run `cuhlumn post push how-we-back-up-sqlite.md`.
3. Read `url` in the output. That address shows the post.

The slug comes from the `slug` field, or from the file name without `.md`.
Without `status: published`, the post is saved as a draft that readers cannot see.
`--publish` and `--draft` override the status in the file.

## Front matter fields

| Field | Default | Rule |
| --- | --- | --- |
| `title` | Required | At most 300 characters |
| `slug` | File name | Lowercase letters, digits, and single hyphens, at most 100 bytes. Not `api`, `og`, `media`, or `tags` |
| `description` | Empty | One or two sentences for search results and share cards |
| `tags` | None | At most 20. Repeated tags are removed without regard to case |
| `kind` | `post` | `post` for dated articles, `page` for pages such as About |
| `status` | `draft` | `draft` or `published` |
| `cover` | None | Image address, such as `/media/<name>.png` |
| `og_image` | None | Share image that replaces the generated card |
| `show_cta` | Site default | `true` or `false` shows or hides the call to action after this post. Without it, the post follows the site |
| `published_at` | First publication | Unix seconds, `2026-05-01`, or `2026-05-01T09:30:00Z`; use it only to import old posts |

Other fields are errors.
The body is required and comes after the closing `---` line.
Raw HTML in the body is allowed.

## Edit a post

`post push` replaces the whole post, so start from the stored version:

1. `cuhlumn post get <slug> --markdown > <slug>.md`
2. Edit the file.
3. `cuhlumn post push <slug>.md`

For a small change, use `post set` and keep the other fields:

```sh
cuhlumn post set <slug> title="A better title"
cuhlumn post set <slug> tags:='["Ops", "SQLite"]'
cuhlumn post set <slug> cover:=null
```

`post push` refuses to turn a published post into a draft.
Add `status: published` to the file, or pass `--draft` when you want to hide the post.

## Other post commands

| Command | Result |
| --- | --- |
| `cuhlumn post list` | All posts and pages without bodies. Filter with `--status draft` or `--kind page` |
| `cuhlumn post get <slug>` | One post as JSON |
| `cuhlumn post publish <slug>` | Makes the post visible |
| `cuhlumn post unpublish <slug>` | Hides the post and keeps it as a draft |
| `cuhlumn post preview <file.md>` | Renders the body to HTML with the table of contents, without saving |
| `cuhlumn post delete <slug> --yes` | Deletes the post |

The first publication time stays when you edit, unpublish, or publish again.

## Images

1. Run `cuhlumn media upload diagram.png`.
2. Use `path` from the output in Markdown or settings: `![Diagram](/media/<name>.png)`.

- Accepted types: PNG, JPEG, WebP, GIF, and AVIF, up to 16 MiB.
- The same image always gets the same name, so a repeated upload is safe.
- `cuhlumn media list` shows the images, and `cuhlumn media delete <name> --yes` deletes one.
- Use the root-relative `path`, not the absolute `url`, in posts and settings.

## Sites

| Command | Result |
| --- | --- |
| `cuhlumn site list` | All sites on the server |
| `cuhlumn site get` | The site of `CUHLUMN_SITE` |
| `cuhlumn site get --input > site.json` | The settings as a file for `site apply` |
| `cuhlumn site apply site.json` | Creates the site, or replaces all its settings; `-` reads stdin |
| `cuhlumn site set field=value ...` | Changes some settings and keeps the others |
| `cuhlumn site delete --yes` | Deletes the site with all its posts and images |

Create a site with a JSON or YAML file:

```json
{
  "title": "Harbor Notes",
  "description": "How a two-person team runs Harbor, a small hosting service.",
  "aliases": ["www.blog.example.com"],
  "author": "Mara Lind",
  "project_name": "Harbor",
  "project_url": "https://harbor.example.com",
  "cta": "Harbor runs your app on a server that you own.",
  "lang": "en",
  "brand": { "accent": "#0e7490", "font": "serif" }
}
```

`site apply` resets omitted fields to their defaults, except the IndexNow key.
Use `site set` for changes to an existing site.

### Site fields

| Field | Meaning |
| --- | --- |
| `title` | Blog name, required, at most 200 characters |
| `description` | Home page text and meta description |
| `aliases` | Other domains that redirect to the site |
| `author`, `author_url` | Author name and absolute address |
| `project_name`, `project_url` | The product link in the header and the call to action after posts |
| `cta` | Call-to-action text above the product button |
| `show_cta` | `true` shows the call to action on posts that do not set `show_cta`. Default `true` |
| `lang` | Language code, such as `en` or `pt-BR` |
| `home` | Slug of a page to show at `/`. The page must exist first |
| `indexnow` | `true` sends IndexNow pings when a post is published. Default `true` |
| `indexnow_key` | IndexNow key. The server makes one for a new site |
| `announcement` | The announcement bar. Change it with `cuhlumn announce` |

### Appearance (`brand`)

| Field | Default | Meaning |
| --- | --- | --- |
| `brand.accent` | `#2563eb` | Accent color, `#rgb` or `#rrggbb`; link colors stay readable automatically |
| `brand.font` | `serif` | `serif` or `sans` for post text. With `font_family`, the fallback stack |
| `brand.font_family` | None | A family name from the font library, such as `"Lora"`. Must exist in the library when it changes |
| `brand.logo` | None | Header and share card image |
| `brand.favicon` | None | Browser icon. Without it, the logo or a generated icon is used |
| `brand.og_image` | None | Share image for the home page |
| `brand.links` | None | Up to 8 header links: `[{"label": "Status", "url": "https://status.harbor.example.com"}]` |
| `brand.head` | Empty | Raw HTML for every page head, such as analytics |
| `brand.footer` | Empty | Footer text, at most 500 characters |
| `brand.credit` | `true` | Shows "Published with Cuhlumn" |
| `brand.palette` | `paper` | `paper`, `slate`, `sepia`, `mono`, or `contrast` |
| `brand.theme` | `system` | `system`, `light`, or `dark` |
| `brand.theme_toggle` | `true` | Shows the light and dark toggle button |
| `brand.width` | `normal` | `narrow`, `normal`, or `wide` text column |
| `brand.home_layout` | `list` | `list`, `compact`, or `cards` |
| `brand.code_theme` | `github` | `github`, `solarized`, `ocean`, or `none` (no highlighting) |
| `brand.card_style` | `dark` | Share card colours: `dark`, `light`, or `accent` |
| `brand.card_meta` | `true` | Shows the date and reading time on share cards |
| `brand.social` | None | Up to 12 profile links, without repetitions: `[{"network": "github", "url": "https://github.com/maralind"}]` |
| `brand.social_in_header` | `false` | Also shows the social links in the header, not only in the footer |

Social networks: `github`, `gitlab`, `codeberg`, `mastodon`, `bluesky`, `x`, `threads`, `linkedin`, `youtube`, `instagram`, `discord`, `telegram`, and `email`.
Each `url` is an absolute `https://` address, at most 500 characters.
For `email`, the `url` is `mailto:` with one address, such as `mailto:mara@harbor.example.com`.

Examples:

```sh
cuhlumn site set brand.accent=#0f766e brand.font=sans
cuhlumn site set brand.logo=/media/<name>.png
cuhlumn site set brand.credit:=false brand.favicon:=null
cuhlumn site set 'brand.links:=[{"label": "Docs", "url": "/docs/"}]'
cuhlumn site set indexnow:=false
cuhlumn site set brand.palette=sepia brand.home_layout=cards
cuhlumn site set brand.code_theme=ocean brand.card_style=light
cuhlumn site set 'brand.social:=[{"network": "github", "url": "https://github.com/maralind"}, {"network": "email", "url": "mailto:mara@harbor.example.com"}]'
```

`field=text` sets a string.
`field:=json` sets a number, a boolean, `null`, a list, or an object.
Quote assignments with spaces or special characters for the shell.

`brand.social:=[...]` replaces the whole list, so include the links that you want to keep.

## Announcement bar

A site can show one announcement bar at the top of every HTML page.

| Command | Result |
| --- | --- |
| `cuhlumn announce show` | Prints the announcement and its status |
| `cuhlumn announce set "<text>" [options]` | Replaces the whole announcement |
| `cuhlumn announce clear` | Removes the announcement |

| Option | Default | Meaning |
| --- | --- | --- |
| `<text>` | Required | Plain text, 1 to 160 characters |
| `--link URL` | None | Address that the link opens |
| `--label TEXT` | `Learn more` | Link text, at most 40 characters. Needs `--link` |
| `--tone T` | `info` | `info`, `accent`, or `warning` |
| `--from TIME` | Now | Start time |
| `--until TIME` | Until cleared | End time, later than `--from` |
| `--no-dismiss` | Off | Removes the close button |

TIME is RFC 3339, such as `2026-11-10T09:00:00Z`, or unix seconds, such as `1794301200`.
`set` does not keep old values: omitted options take their defaults.

```sh
cuhlumn announce set "Maintenance on Sunday." --tone warning \
  --link https://harbor.example.com/maintenance --label Details \
  --from 2026-11-10T09:00:00Z --until 2026-11-15T18:00:00Z
```

Every `announce` command prints `{"status": ..., "announcement": ...}`.
`status` is `off` (no announcement), `scheduled`, `live`, or `ended`.
`announcement` is `null` when the status is `off`.

## Font library

The font library belongs to the server, not to a site.
The `font` commands need the server address and the token, but not the site.

```sh
cuhlumn font import 'https://fonts.googleapis.com/css2?family=Lora:ital,wght@0,400;0,700;1,400'
cuhlumn site set brand.font_family=Lora
cuhlumn font list
cuhlumn font upload HarborSans-Regular.ttf
cuhlumn font delete <name> --yes
```

- `font list` prints the fonts as JSON, with `name`, `families`, `source`, `size`, and `created_at`.
- `font import <url>` imports fonts from a Google Fonts specimen page or a `css2` or `css` stylesheet address, and prints the new fonts.
- A specimen page, such as `https://fonts.google.com/specimen/Lora`, imports only the regular face.
- To import bold and italic faces, use a `css2` address with `ital,wght@0,400;0,700;1,400`.
- `font upload <file>` uploads one TTF, OTF, or TTC file of up to 20 MB, and prints the font.
- `font delete <name> --yes` deletes a font. Without `--yes`, the command refuses.
- A blog uses a library family through `brand.font_family`.
- Share images use library fonts for characters that Inter does not have.
- The server serves the fonts publicly. Use only fonts whose license allows web use, such as Google Fonts.

## Home page

1. Push a page: front matter with `kind: page` and `status: published`, for example slug `home`.
2. Run `cuhlumn site set home=home`.

The home page shows the page and then the latest posts.
The server refuses to delete the home page or to change its kind.
Run `cuhlumn site set home:=null` first.

## Rules to remember

- Readers see a change at once on the server, but a CDN or browser can show the old page for up to 1 minute.
- Pages appear in the header navigation. Posts appear in the post list, the feed, and tag pages.
- Raw HTML in posts and `brand.head` goes to readers without changes.
- `cuhlumn skill` prints this file.
- `cuhlumn install-skill -a <agent> -y` installs this skill into another agent, for example `-a codex`. Add `-g` to install it for all projects of the user.
