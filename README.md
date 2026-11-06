# Cuhlumn

<p align="center">
  <img src="assets/logo.png" width="160" alt="Cuhlumn: an orange cat">
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

**Documentation**: [`docs/`](docs/content/index.md)

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
Put Cloudflare or any TLS proxy in front of it.

| Feature | Result |
| --- | --- |
| Multi-domain hosting | Serves many blogs, each with aliases that redirect to the main domain |
| Drafts | Keeps unpublished posts out of pages, feeds, and sitemaps |
| Share images | Generates a 1200×630 card for each blog and each post |
| IndexNow | Notifies Bing, Yandex, and other engines when a post is published |
| Management console | Edits sites, posts, appearance, and images in the browser |
| Agent skill | Installs into Claude Code, Codex, Cursor, and other agents |

---

## Installation

### Server with Docker

Build the image and start a container:

```bash
docker build -t cuhlumn .
docker run -d --name cuhlumn -p 8080:8080 -v cuhlumn-data:/data \
  -e CUHLUMN_TOKEN="$(openssl rand -hex 32)" \
  -e CUHLUMN_CONSOLE_HOST=admin.example.com \
  cuhlumn
```

Point every blog domain and the console domain at the server.
Put a TLS proxy in front of the server that keeps the `Host` header.
[Cloudflare](docs/content/operations/cloudflare.md) is the recommended proxy because it also caches the pages.

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

Install the `cuhlumn` command-line client:

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

[Read the full quick-start guide](docs/content/getting-started/quickstart.md).

---

## Core Features

### Publishing

A post is a Markdown file with YAML front matter.
The slug comes from the file name or the `slug` field.
New posts are drafts until you publish them.

Markdown supports tables, footnotes, task lists, strikethrough, smart punctuation, and custom heading ids.
Raw HTML passes through, so give the admin token only to trusted writers.

A blog can show one of its pages as the home page, with the latest posts below it.

[Read about posts and drafts](docs/content/platform/posts.md).

### Search and AI Discovery

| Signal | Where |
| --- | --- |
| JSON-LD | `Blog` or `WebSite`, `BlogPosting` or `WebPage`, `BreadcrumbList`, `CollectionPage` |
| Social previews | Open Graph and Twitter tags with a generated share image |
| Crawler files | `sitemap.xml`, `feed.xml`, `robots.txt`, `llms.txt`, `llms-full.txt` |
| Markdown copies | `/{slug}.md` for every published entry |
| IndexNow | A notification for each published post |

[Read about discovery](docs/content/platform/discovery.md).

### Appearance

Each blog sets its accent color, text font, logo, favicon, share image, header links, extra head HTML, and footer text.
Link colors are adjusted until they meet WCAG AA contrast in the light and the dark theme.

[Read about appearance](docs/content/platform/appearance.md).

### Performance

The server renders a page once and keeps it in memory with an ETag and a gzip copy.
A change to a blog clears only the cache of that blog.
A post page transfers about 5 KiB.

[Read about caching](docs/content/platform/caching.md).

---

## Configuration

The server reads environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `CUHLUMN_TOKEN` | none | Admin token, at least 32 bytes; without it the API and the console stay off |
| `CUHLUMN_CONSOLE_HOST` | none | Domain of the management console |
| `CUHLUMN_DATA` | `./data` | Folder that holds the database and the uploaded images |
| `CUHLUMN_SCHEME` | `https` | Scheme in generated addresses and the cookie `Secure` flag |
| `CUHLUMN_INDEXNOW` | `on` | `off` disables IndexNow notifications |
| `CUHLUMN_FONTS` | none | Extra fonts for share images |
| `PORT` | `8080` | Listen port |
| `RUST_LOG` | `info` | Log filter |

[Read the configuration guide](docs/content/operations/configuration.md).

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
| `cuhlumn install-skill` | Install the agent skill |

[Read the complete CLI reference](docs/content/cli/index.md).

---

## API Quick Reference

Admin requests send `Authorization: Bearer <token>` to any domain of the server.
Errors are JSON objects with a `code`, a `message`, and the invalid `field`.

| Route | Purpose |
| --- | --- |
| `GET /api/health` | Server status and version |
| `GET /api/sites` | All blogs |
| `GET`, `PUT`, `DELETE /api/sites/{host}` | One blog |
| `GET /api/sites/{host}/posts` | Every post and page, including drafts |
| `GET`, `PUT`, `DELETE /api/sites/{host}/posts/{slug}` | One post or page |
| `GET`, `POST /api/sites/{host}/media` | List or upload images |
| `DELETE /api/sites/{host}/media/{name}` | Delete an image |
| `POST /api/sites/{host}/preview` | Render Markdown without saving |

[Read the API reference](docs/content/reference/api.md).

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
