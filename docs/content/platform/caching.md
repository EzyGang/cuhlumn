# Caching

The server renders a page once and keeps it in memory until the next write to the same site.

## Page cache

| Cached | Not cached |
| --- | --- |
| HTML pages, crawler files, the favicon, the key file, share images | Redirects, 404 answers, media files, errors |

- The cache key is the canonical domain and the decoded path. The query string is not part of the key.
- A cached page is served without a database query.
- The cache has no size limit. A write empties the cache of one site.

## Invalidation

These API calls empty the cache of the site:

- Site update or delete
- Post save or delete
- Media upload or delete

A page that was being rendered during a write is not stored.
Other sites keep their cache.

These API calls empty the cache of every site:

- A server settings update that changes the scheme, because pages contain absolute addresses
- Font upload, Google Fonts import, or font deletion, because share images use the fonts

## Announcement boundaries

When a site has an announcement with a start or end time in the future, each cached HTML page expires at the next of these times.
The next request after that time renders the page again, so the bar appears or disappears on time.
Crawler files and share images do not contain the bar, so they do not expire.
See [Announcements](announcements.md#caching).

## HTTP headers

| Header | Value |
| --- | --- |
| `ETag` | The first 16 hexadecimal characters of the SHA-256 hash of the body, in quotes |
| `Cache-Control` | `public, max-age=N`, with N from [Public pages](public-pages.md#addresses) |
| `Vary` | `accept-encoding` |
| `Content-Encoding` | `gzip`, when the client accepts it |

A request with a matching `If-None-Match` gets `304` without a body.
The server compresses bodies larger than 1024 bytes, except PNG images.

## Browser and CDN caches

HTML pages have `max-age=60`.
After you unpublish or change a post, a browser or a CDN can show the old page for up to one minute.
The server itself shows the change immediately.

We recommend [Cloudflare](../operations/cloudflare.md) as a second cache layer in front of the server.
Cloudflare needs one cache rule to cache HTML pages and crawler files.
