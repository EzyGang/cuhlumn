# Cuhlumn documentation

Cuhlumn is a blogging platform that serves many blogs, each on its own domain, from one server.
It renders every page as server-side HTML with structured data, so search engines and AI assistants can find, read, and cite it.
Writers, scripts, and AI agents publish Markdown posts through an authenticated HTTP API or a web management console.
The server is a single Rust binary that keeps its data in one SQLite file and one media folder.

## Start here

- [Getting started](getting-started/index.md) explains installation and the first published post.
- [Sites and domains](platform/sites.md) explains how one server hosts many blogs.
- [Posts, pages, and drafts](platform/posts.md) explains the content model.
- [Appearance](platform/appearance.md) lists the look settings of a blog.
- [Management console](console/index.md) describes the web interface.
- [Command-line client](cli/index.md) publishes Markdown files from scripts and AI agents.
- [Configuration](operations/configuration.md) lists the server settings.
- [HTTP API](reference/api.md) describes every route, object, and error.
- [Under the hood](concepts/how-it-works.md) explains how a request becomes a page.

## Supported features

| Capability | Support |
| --- | --- |
| Blogs | Many blogs on one server, each with a canonical domain and alias domains |
| Content | Markdown posts and pages, drafts, tags, cover images, a custom home page |
| Markdown | Tables, footnotes, strikethrough, task lists, smart punctuation, heading ids, raw HTML |
| Public files | HTML pages, Markdown copies, Atom feed, sitemap, `robots.txt`, `llms.txt`, `llms-full.txt` |
| Structured data | `Blog`, `WebSite`, `BlogPosting`, `WebPage`, `BreadcrumbList`, and `CollectionPage` in JSON-LD |
| Share images | Generated 1200×630 PNG cards for each site and each post |
| Search engine notification | IndexNow pings when a post is published |
| Appearance | Accent color, serif or sans-serif text, logo, favicon, share image, header links, head HTML, footer, credit line |
| Themes | Light and dark themes with a reader toggle |
| Media | PNG, JPEG, WebP, GIF, and AVIF uploads of up to 16 MiB |
| Analytics | Page views, daily visitors, top pages, referrers, devices, and browsers, without cookies |
| Management | HTTP API with a bearer token, `cuhlumn` command-line client with an agent skill, web console with a session cookie |
| Deployment | One binary, one SQLite file, Docker image |
| Lighthouse | 100 for performance, accessibility, best practices, and SEO on home pages, posts, and pages, on mobile and desktop |

## What Cuhlumn leaves out

Cuhlumn has no comments, newsletters, user accounts, or themes beyond the appearance settings.
One admin token controls every blog on the server.
