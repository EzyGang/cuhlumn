# Posts, pages, and drafts

Each site contains entries.
An entry is a post or a page, and it is a draft or published.

## Kinds

| Kind | Appears in | Date shown |
| --- | --- | --- |
| `post` | The post list, the feed, tag pages, "More posts", the sitemap, `llms.txt` | Yes |
| `page` | The header navigation, the sitemap, `llms.txt` | No |

## Status

| Status | Visible to readers |
| --- | --- |
| `draft` | No. Every public address of the entry returns `404`, and no list or file contains it. |
| `published` | Yes |

A new entry is a draft unless the request sets `"status": "published"`.
An edit to a published entry goes live when you save it.
To hide an entry again, save it with `"status": "draft"`.

## Publication time

The server sets `published_at` when an entry is published for the first time.

- Edits do not change the publication time.
- Unpublishing and publishing again keep the first publication time.
- You can set `published_at` explicitly to import older posts. The value is Unix time in seconds.

`updated_at` changes on every save.
The post page shows "updated" when `updated_at` is more than one day after `published_at`.

## Slugs

The slug comes from the address of the `PUT` request.
A slug contains lowercase letters, digits, and single hyphens, with a maximum of 100 bytes.
`api`, `og`, `media`, and `tags` are reserved.
You cannot rename an entry.

## Tags

A post can have at most 20 tags.
The server removes repeated tags without regard to case and keeps the first spelling.
Each tag has a page at `/tags/{tag}/`, where `{tag}` is the lowercase tag with other characters replaced by hyphens.
Tag pages tell search engines not to index them.

## Home page

Set the site `home` to the slug of a page to show that page at `/`.

- The page must exist and must be a page, otherwise the site update returns `400`.
- The home page appears at `/` with the heading "Latest posts" and the post list below it.
- `/{home}/` redirects to `/`.
- The home page is not in the header navigation or the sitemap.
- You cannot delete the home page or change its kind. Choose another home page first.

If the home page is a draft, `/` shows the normal post list.

## Markdown

The body is Markdown with these extensions:

- Tables
- Footnotes
- Strikethrough
- Task lists
- Smart punctuation
- Heading attributes, such as `## Setup {#setup}`

Raw HTML passes through without changes, so only give the token to writers you trust.
See [Public pages](public-pages.md) for how the server renders the body.

## Order

Lists show entries by publication time, newest first.
Entries without a publication time use their update time.
