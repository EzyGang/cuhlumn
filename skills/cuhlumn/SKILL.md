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
`cuhlumn health` checks the connection without a token.
Never print the token or write it into a file in a repository.

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
| `slug` | File name | Lowercase letters, digits, and single hyphens, at most 100 bytes; not `api`, `og`, `media`, or `tags` |
| `description` | Empty | One or two sentences for search results and share cards |
| `tags` | None | At most 20; repeated tags are removed without regard to case |
| `kind` | `post` | `post` for dated articles, `page` for pages such as About |
| `status` | `draft` | `draft` or `published` |
| `cover` | None | Image address, such as `/media/<name>.png` |
| `og_image` | None | Share image that replaces the generated card |
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
| `cuhlumn post list` | All posts and pages without bodies; filter with `--status draft` or `--kind page` |
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
| `project_name`, `project_url` | The product link in the header and after each post |
| `cta` | Call-to-action text after each post |
| `lang` | Language code, such as `en` or `pt-BR` |
| `home` | Slug of a page to show at `/`; the page must exist first |
| `indexnow_key` | IndexNow key; the server makes one for a new site |

### Appearance (`brand`)

| Field | Default | Meaning |
| --- | --- | --- |
| `brand.accent` | `#2563eb` | Accent color, `#rgb` or `#rrggbb`; link colors stay readable automatically |
| `brand.font` | `serif` | `serif` or `sans` for post text |
| `brand.logo` | None | Header and share card image |
| `brand.favicon` | None | Browser icon; without it the logo or a generated icon is used |
| `brand.og_image` | None | Share image for the home page |
| `brand.links` | None | Up to 8 header links: `[{"label": "Status", "url": "https://status.harbor.example.com"}]` |
| `brand.head` | Empty | Raw HTML for every page head, such as analytics |
| `brand.footer` | Empty | Footer text, at most 500 characters |
| `brand.credit` | `true` | Shows "Published with Cuhlumn" |

Examples:

```sh
cuhlumn site set brand.accent=#0f766e brand.font=sans
cuhlumn site set brand.logo=/media/<name>.png
cuhlumn site set brand.credit:=false brand.favicon:=null
cuhlumn site set 'brand.links:=[{"label": "Docs", "url": "/docs/"}]'
```

`field=text` sets a string.
`field:=json` sets a number, a boolean, `null`, a list, or an object.
Quote assignments with spaces or special characters for the shell.

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
