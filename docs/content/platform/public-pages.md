# Public pages

The server renders every public page as HTML on the server.
A reader needs no JavaScript to read a post.
The only scripts are two small inline scripts for the theme toggle.

## Addresses

| Path | Contents | Type | `max-age` |
| --- | --- | --- | --- |
| `/` | Home page | HTML | 60 |
| `/{slug}/` | Post or page | HTML | 60 |
| `/{slug}.md` | Markdown copy of an entry | `text/markdown` | 300 |
| `/tags/{tag}/` | Posts with one tag | HTML | 60 |
| `/feed.xml` | Atom feed | `application/atom+xml` | 300 |
| `/sitemap.xml` | Sitemap | `application/xml` | 300 |
| `/robots.txt` | Crawler rules | `text/plain` | 300 |
| `/llms.txt` | Index for AI assistants | `text/plain` | 300 |
| `/llms-full.txt` | Full text for AI assistants | `text/plain` | 300 |
| `/favicon.svg` | Generated icon: the site initial on an accent disc | `image/svg+xml` | 86400 |
| `/og.png` | Share image of the site | `image/png` | 86400 |
| `/og/{slug}.png` | Share image of an entry | `image/png` | 86400 |
| `/media/{name}` | Uploaded image | Image type | 31536000, immutable |
| `/{indexnow_key}.txt` | IndexNow key file | `text/plain` | 300 |

Public paths accept `GET` and `HEAD` only.
Other methods return `405`.

## Redirects

All redirects use status `301`.

| Request | Target |
| --- | --- |
| Any path on an alias domain | The same path and query on the canonical domain |
| `/{slug}` | `/{slug}/`, when the entry exists |
| `/tags/{tag}` | `/tags/{tag}/`, when the tag exists |
| `/{home}` or `/{home}/` | `/` |
| `/favicon.ico` | `/favicon.svg` |

The slug and tag redirects keep the query string.

## Home page

Without a home page, `/` shows the site title, the description, and all published posts.
With a home page, `/` shows the page and then all posts under "Latest posts".
Each list item shows the date, the title, and a summary.
The summary is the description, or the first 155 characters of the text.
A blog without published posts shows "No posts yet." in place of the list.

## Post page

A post page contains these parts:

1. A reading progress bar at the top of the window, in browsers that support scroll-driven animations
2. A breadcrumb link to the home page
3. The title and the description
4. The date, the "updated" date when it differs by more than one day, the reading time, and the author
5. The cover image
6. The table of contents "Contents", when the body has three or more second- and third-level headings
7. The body
8. The tags
9. The call to action, when the site has a `project_url` and the entry shows it. See [Call to action](posts.md#call-to-action)
10. "More posts": up to three other posts that share a tag, or the three latest posts

A page has the same layout without the progress bar, the date line, and "More posts".

## Rendering details

- Every heading gets an `id`. A `{#id}` attribute wins. Otherwise the id comes from the heading text. Repeated ids get `-2`, `-3`, and so on.
- Second- and third-level headings get a `#` anchor link. The link shows when the pointer is on the heading.
- A table with two columns becomes a definition list. Wider tables become a table whose rows turn into cards on screens narrower than 40rem.
- Images in the body load lazily. The cover image loads first.
- The reading time is the word count divided by 220, with a minimum of one minute.
- Dates use UTC and the format "October 27, 2026".

## 404 page

An unknown path returns `404` with the heading "Page not found" and the five latest posts.
The 404 page tells search engines not to index it.

## Footer

The footer shows the site title and description, links to the feed, the sitemap, and `llms.txt`, the `footer` text, and the credit line.
