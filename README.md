<p align="center">
  <picture>
    <source media="(max-width: 600px)" srcset="assets/logo-square.png">
    <img src="assets/logo-wide.png" width="640" alt="Cuhlumn: a grumpy grey cat. Blogs, cuh.">
  </picture>
</p>

<p align="center">
<a href="https://github.com/EzyGang/cuhlumn/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT">
</a>
<img src="https://img.shields.io/badge/rust-2024%20edition-orange" alt="Rust 2024 edition">
</p>

---

<h3 align="center">One server, many blogs</h3>

<p align="center">
  Write Markdown from an agent, a script, or a browser. Readers, crawlers, and AI assistants get plain HTML.
</p>

**Cuhlumn** is a self-hosted blogging platform for teams that run more than one blog.
One process serves every blog from its own domain.
Every page is rendered on the server with structured data, feeds, and machine-readable copies.

Publishing does not need a browser.
Posts arrive through an HTTP API, the `cuhlumn` command-line client, or the management console.

**Documentation**: https://cuhlumn.ezygang.digital

**Source Code**: https://github.com/EzyGang/cuhlumn

**Issues**: https://github.com/EzyGang/cuhlumn/issues

---

## Table of Contents

- [Why Cuhlumn?](#why-cuhlumn)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Features](#core-features)
  - [Publishing](#publishing)
  - [Search and AI Discovery](#search-and-ai-discovery)
  - [Appearance](#appearance)
  - [Performance](#performance)
- [Configuration](#configuration)
- [CLI Quick Reference](#cli-quick-reference)
- [API Quick Reference](#api-quick-reference)
- [Contributing](#contributing)
- [License](#license)

---

## Why Cuhlumn?

**🗂️ Many blogs, one deployment**
A product blog, a changelog, and a personal site can share one server.
Each blog keeps its own domain, look, posts, and images.

**🤖 Built for agents**
The CLI reads Markdown files with front matter and prints JSON.
An agent skill teaches coding agents to write, publish, and maintain posts.

**🔎 Readable by machines**
Pages need no JavaScript.
Each page carries JSON-LD, Open Graph tags, and a canonical address.
Each blog also serves a sitemap, an Atom feed, `llms.txt`, and a Markdown copy of every entry.

**⚡ Fast by default**
Rendered pages stay in memory until the next change.
Home pages, posts, and pages score 100 in all four Lighthouse categories.

**📦 Simple to operate**
Back up one folder.
Run one container.
The first visit to the console sets the admin token, so no secret must go into the environment.
Put Cloudflare or any TLS proxy in front of it.

| Feature | Result |
| --- | --- |
| Multi-domain hosting | Serves many blogs, each with aliases that redirect to the main domain |
| Drafts | Keeps unpublished posts out of pages, feeds, and sitemaps |
| Share images | Generates a 1200×630 card for each blog and each post, in three styles |
| IndexNow | Notifies Bing, Yandex, and other engines when a post is published, per blog |
| Appearance | Five palettes, three widths, three home layouts, social icons, and code highlighting |
| Announcement bar | Shows a scheduled message above every page of a blog |
| Management console | Edits sites, posts, appearance, images, fonts, and server settings in the browser |
| Agent skill | Installs into Claude Code, Codex, Cursor, and other agents |

---

## Installation

### Server with Docker Compose

Download `docker-compose.yml` and `.env.example` from this repository, then start the server:

```bash
cp .env.example .env
docker compose up -d
```

Compose starts the image `ghcr.io/ezygang/cuhlumn` and publishes the port on `127.0.0.1:8080`.
The image checks itself with `cuhlumn-server healthcheck`.

On the first launch without `CUHLUMN_TOKEN`, open the console and set the admin token on the **Setup** page.

> [!WARNING]
> In setup mode, the first visitor of the console sets the token and controls the server.
> Set `CUHLUMN_TOKEN` in `.env`, or keep the port private until you complete the setup.

Point every blog domain and the console domain at the server.
Put a TLS proxy in front of the server that keeps the `Host` header.
[Cloudflare](https://cuhlumn.ezygang.digital/operations/cloudflare/) is the recommended proxy because it also caches the pages.
[Read the installation guide](https://cuhlumn.ezygang.digital/getting-started/installation/).

### Server from source

You need stable Rust with edition 2024, Node.js 22.12 or later, and pnpm 11.

```bash
git clone https://github.com/EzyGang/cuhlumn.git
cd cuhlumn
pnpm install
pnpm build
cargo build --release -p cuhlumn-server
```

The release binary embeds the console, so build the console first.

### CLI

Download the `cuhlumn` command-line client for your system from the [latest release](https://github.com/EzyGang/cuhlumn/releases/latest).
Each release has archives for Linux, macOS, and Windows on x86_64 and arm64, and a `SHA256SUMS` file.

```bash
curl -fsSL https://github.com/EzyGang/cuhlumn/releases/latest/download/cuhlumn-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo install cuhlumn-x86_64-unknown-linux-gnu/cuhlumn /usr/local/bin/
```

You can also build it with Cargo:

```bash
cargo install --git https://github.com/EzyGang/cuhlumn --locked cuhlumn-cli
```

### Agent skill

Install the skill from this repository with npm or pnpm.
The repository also contains skills for contributors, so `--skill cuhlumn` selects only the user skill.

```bash
npx skills@latest add EzyGang/cuhlumn --skill cuhlumn
pnpm dlx skills@latest add EzyGang/cuhlumn --skill cuhlumn
```

The CLI can also install the skill that matches its own version:

```bash
cuhlumn install-skill
```

---

## Quick Start

Connect the CLI to your server:

```bash
export CUHLUMN_SERVER=https://blog.example.com
export CUHLUMN_TOKEN=<your admin token>
export CUHLUMN_SITE=blog.example.com
```

Create a blog:

```bash
echo '{"title": "Harbor Notes", "description": "How a two-person team runs Harbor, a small hosting service."}' \
  | cuhlumn site apply -
```

Write `sqlite-backups.md`:

```markdown
---
title: How we back up SQLite
description: Our nightly routine for one database file.
tags: [Databases]
---

We back up **one SQLite file** every night.
```

Save it as a draft, preview it, and publish it:

```bash
cuhlumn post push sqlite-backups.md
cuhlumn post preview sqlite-backups.md
cuhlumn post publish sqlite-backups
```

The post is now at `https://blog.example.com/sqlite-backups/`.

[Read the full quick-start guide](https://cuhlumn.ezygang.digital/getting-started/quickstart/).

---

## Core Features

### Publishing

A post is a Markdown file with YAML front matter.
The slug comes from the file name or the `slug` field.
New posts are drafts until you publish them.

Markdown supports tables, footnotes, task lists, strikethrough, smart punctuation, and custom heading ids.
Raw HTML passes through, so give the admin token only to trusted writers.

A blog can show one of its pages as the home page, with the latest posts below it.

[Read about posts and drafts](https://cuhlumn.ezygang.digital/platform/posts/).

### Search and AI Discovery

| Signal | Where |
| --- | --- |
| JSON-LD | `Blog` or `WebSite`, `BlogPosting` or `WebPage`, `BreadcrumbList`, `CollectionPage` |
| Social previews | Open Graph and Twitter tags with a generated share image |
| Crawler files | `sitemap.xml`, `feed.xml`, `robots.txt`, `llms.txt`, `llms-full.txt` |
| Markdown copies | `/{slug}.md` for every published entry |
| IndexNow | A notification for each published post, when the blog has IndexNow on |
| Profile links | Social links in the JSON-LD `sameAs` list of the publisher |

[Read about discovery](https://cuhlumn.ezygang.digital/platform/discovery/).

### Appearance

Each blog sets its accent color, text font, logo, favicon, share image, header links, extra head HTML, and footer text.
It also selects a palette, a theme mode, a column width, a home page layout, a code theme, and a share image style.
Social profile links show as icons in the footer, and optionally in the header.
Link and code colors are adjusted until they meet WCAG AA contrast in the light and the dark theme.

Each blog can show an announcement bar with a start time, an end time, and a link.

Share images use Inter first, then the system fonts and the font library.
Upload font files or import them from Google Fonts to add characters for other scripts.

[Read about appearance](https://cuhlumn.ezygang.digital/platform/appearance/).
[Read about announcements](https://cuhlumn.ezygang.digital/platform/announcements/).
[Read about share images](https://cuhlumn.ezygang.digital/platform/share-images/).

### Performance

The server renders a page once and keeps it in memory with an ETag and a gzip copy.
A change to a blog clears only the cache of that blog.
A post page transfers about 5 KiB.

[Read about caching](https://cuhlumn.ezygang.digital/platform/caching/).

---

## Configuration

The server reads a few environment variables.
You change the admin token, the console domain, and the scheme on the **Server settings** page of the console.
The three override variables win over the console values, and the console then shows those fields as locked.

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | Listen port for the blogs |
| `CUHLUMN_ADMIN_ADDR` | none | Optional address of a private admin listener for the console and the API, such as `0.0.0.0:8081` |
| `CUHLUMN_DATA` | `./data` | Folder that holds the database, the uploaded images, and the fonts |
| `RUST_LOG` | `info` | Log filter |
| `CUHLUMN_TOKEN` | none | Optional override of the admin token, at least 32 bytes |
| `CUHLUMN_CONSOLE_HOST` | none | Optional override of the console domain |
| `CUHLUMN_SCHEME` | `https` | Optional override of the scheme in generated addresses and, without an admin listener, the cookie `Secure` flag |

With `CUHLUMN_ADMIN_ADDR`, the `PORT` listener serves only the blogs and `/api/health`.
The admin port works with HTTPS through a TLS proxy, or with plain HTTP on a private network that encrypts traffic, such as NetBird.
Set `CUHLUMN_SERVER` of the CLI to the admin address.

IndexNow is a setting of each site, and fonts for share images are uploaded in the console.

[Read the configuration guide](https://cuhlumn.ezygang.digital/operations/configuration/).

---

## CLI Quick Reference

| Command | Purpose |
| --- | --- |
| `cuhlumn site apply <file>` | Create a blog or replace its settings from JSON or YAML |
| `cuhlumn site set <field=value>...` | Change some settings and keep the others |
| `cuhlumn post push <file.md>` | Save a post from a Markdown file |
| `cuhlumn post get <slug> --markdown` | Write a post back as a Markdown file |
| `cuhlumn post publish <slug>` | Make a post visible |
| `cuhlumn post unpublish <slug>` | Hide a post and keep it as a draft |
| `cuhlumn media upload <image>` | Upload an image and print its `/media/` path |
| `cuhlumn announce set "<text>"` | Show an announcement bar on every page |
| `cuhlumn announce clear` | Remove the announcement bar |
| `cuhlumn install-skill` | Install the agent skill |

[Read the complete CLI reference](https://cuhlumn.ezygang.digital/cli/).

---

## API Quick Reference

Admin requests send `Authorization: Bearer <token>` to any domain of the server.
Before the first setup, admin routes return `503` with the code `needs_setup`.
Errors are JSON objects with a `code`, a `message`, and the invalid `field`.

| Route | Purpose |
| --- | --- |
| `GET /api/health` | Server status, version, and the setup flag |
| `POST /api/setup` | Set the first admin token |
| `GET`, `PUT /api/settings` | Console host and scheme |
| `PUT /api/settings/token` | Change the admin token |
| `GET`, `POST /api/fonts` | List or upload share image fonts |
| `POST /api/fonts/google` | Import fonts from Google Fonts |
| `DELETE /api/fonts/{name}` | Delete a font |
| `GET /api/sites` | All blogs |
| `GET`, `PUT`, `DELETE /api/sites/{host}` | One blog |
| `GET /api/sites/{host}/posts` | Every post and page, including drafts |
| `GET`, `PUT`, `DELETE /api/sites/{host}/posts/{slug}` | One post or page |
| `GET`, `POST /api/sites/{host}/media` | List or upload images |
| `DELETE /api/sites/{host}/media/{name}` | Delete an image |
| `POST /api/sites/{host}/preview` | Render Markdown without saving |

[Read the API reference](https://cuhlumn.ezygang.digital/reference/api/).

---

## Contributing

Open a [GitHub issue](https://github.com/EzyGang/cuhlumn/issues) to discuss a large change.
Then open a pull request.

Run these checks before you submit the pull request:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
pnpm validate
pnpm test
```

`AGENTS.md` describes the code rules for Rust, the console, and written text.

---

## License

Cuhlumn is available under the [MIT License](LICENSE).
