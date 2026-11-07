# Search and AI discovery

Cuhlumn writes the signals that search engines and AI assistants use to find, understand, and cite a blog.

## Meta tags

Every page has these tags:

- `<title>` and `<meta name="description">`
- `<link rel="canonical">` with the absolute address on the canonical domain, except on the 404 page
- `robots`: `index, follow, max-image-preview:large`, or `noindex, follow` on tag pages and the 404 page
- Open Graph: `og:type`, `og:site_name`, `og:title`, `og:description`, `og:url`, `og:image`, `og:image:alt`
- Twitter: `twitter:card` with `summary_large_image`, `twitter:title`, `twitter:description`, `twitter:image`
- A link to the Atom feed

Posts also have `article:published_time`, `article:modified_time`, one `article:tag` for each tag, and a link to the Markdown copy.

## Structured data

| Page | JSON-LD |
| --- | --- |
| Home page | `Blog`, or `WebSite` when the site has a home page, with up to 30 recent posts in `blogPost` |
| Post | `BlogPosting` and `BreadcrumbList` |
| Page | `WebPage` and `BreadcrumbList` |
| Tag page | `CollectionPage` |

Posts include the headline, description, address, image, language, author, publisher, dates, word count, and tags.
The author is a `Person` when the site has an author.
Otherwise the author is the publisher, an `Organization` with the site title and logo.

The publisher gets a `sameAs` list with the profile addresses from the `social` links of the site.
Email links are not in the list.
A site without profile links has no `sameAs` list.
See [Appearance](appearance.md#social-links).

## Atom feed

`/feed.xml` contains the 50 latest posts with their full HTML.
Pages are not in the feed.
Root-relative addresses in the content become absolute addresses.

## Sitemap

`/sitemap.xml` lists the home page and every published post and page, with the update time.
The home page entry is not repeated, and tag pages are not listed.

## robots.txt

```text
User-agent: *
Allow: /
Disallow: /api/

Sitemap: https://blog.example.com/sitemap.xml
```

## llms.txt

[`llms.txt`](https://llmstxt.org) is an index for AI assistants:

1. The site title as a heading
2. The description as a quote
3. A line about the product, when the site has a `project_url`
4. A `## Posts` section and a `## Pages` section with links to the Markdown copies and summaries

`llms-full.txt` contains `llms.txt` and then the Markdown copy of every entry.

## Markdown copies

`/{slug}.md` returns an entry as Markdown:

```markdown
# How we back up SQLite

> Our nightly routine for one database file.

Published October 27, 2026 on Harbor Notes: https://blog.example.com/sqlite-backups/

We back up **one SQLite file** every night.
```

## IndexNow

When a post is saved with `published` status and the site has `indexnow` set to `true`, the server sends the post address and the home address to `https://api.indexnow.org/indexnow`.
Bing, Yandex, and the other IndexNow engines receive the notification.
Google reads the sitemap.

- Each site has its own key, served at `/{key}.txt`.
- A new site gets a random key of 32 hexadecimal characters.
- A site update without `indexnow_key` keeps the current key.
- `indexnow` is on by default. Set it to `false` to stop the notifications of one site.
- The console shows the **IndexNow** switch next to the key in the **Domains** section.
- The key file stays available when `indexnow` is `false`.

The notification runs in the background.
The API answer does not wait for it, and a failure only goes to the server log.
