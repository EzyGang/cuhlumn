# Posts

## Routes

| Method | Path | Success | Errors |
| --- | --- | --- | --- |
| `GET` | `/api/sites/{host}/posts` | `200` array of `Post`, drafts included, newest first | `400`, `404` |
| `GET` | `/api/sites/{host}/posts/{slug}` | `200` `Post` | `400`, `404` |
| `PUT` | `/api/sites/{host}/posts/{slug}` | `200` `PostSaved` | `400`, `404`, `409`, `413`, `415` |
| `DELETE` | `/api/sites/{host}/posts/{slug}` | `204` | `400`, `404`, `409` |

`PUT` creates the entry or replaces all its fields.

## `PostInput`

The body of `PUT /api/sites/{host}/posts/{slug}`.
Unknown fields return `400`.

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `title` | String | Required | Trimmed, not blank, at most 300 characters |
| `description` | String | `""` | No control characters |
| `body` | String | Required | Markdown, not blank, no control characters except tab and line breaks |
| `tags` | Array of [`Tag`](value-types.md#tag) | `[]` | At most 20 after case-insensitive deduplication |
| `kind` | `"post"` or `"page"` | `"post"` | |
| `status` | `"draft"` or `"published"` | `"draft"` | |
| `cover` | [`LinkUrl`](value-types.md#linkurl) or `null` | `null` | |
| `og_image` | `LinkUrl` or `null` | `null` | |
| `show_cta` | Boolean or `null` | `null` | `null` follows the site `show_cta`. `true` always shows the call to action, `false` never shows it |
| `published_at` | Integer or `null` | `null` | From 0 to 253402300799 (the end of year 9999) |

## `published_at`

The server resolves the publication time in this order:

1. The `published_at` value of the request
2. The stored value
3. The current time, if `status` is `published`
4. `null`

The first publication time therefore stays across edits and unpublishing.
The API cannot clear a stored publication time.

## `Post`

| Field | Type |
| --- | --- |
| `slug` | `Slug` |
| `title` | String |
| `description` | String |
| `body` | String |
| `tags` | Array of `Tag` |
| `kind` | `"post"` or `"page"` |
| `status` | `"draft"` or `"published"` |
| `cover` | `LinkUrl` or `null` |
| `og_image` | `LinkUrl` or `null` |
| `show_cta` | Boolean or `null` |
| `published_at` | Integer or `null` |
| `updated_at` | Integer |
| `url` | String: `{scheme}://{host}/{slug}/`, or `{scheme}://{host}/` for the home page |

## `PostSaved`

| Field | Type |
| --- | --- |
| `url` | String, the same as `post.url` |
| `post` | `Post` |

## Home page rules

| Request | Result |
| --- | --- |
| `PUT` the home page with a `kind` other than `page` | `409` |
| `DELETE` the home page | `409` |

## Side effects

- Each `PUT` and `DELETE` empties the page cache of the site.
- A `PUT` with `status` `published` sends an IndexNow notification when the site has `indexnow` set to `true`. See [IndexNow](../platform/discovery.md#indexnow).

## Example

```sh
curl -X PUT https://blog.example.com/api/sites/blog.example.com/posts/about \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "About", "body": "We build Example.", "kind": "page", "status": "published"}'
```

```json
{
  "url": "https://blog.example.com/about/",
  "post": {
    "slug": "about",
    "title": "About",
    "description": "",
    "body": "We build Example.",
    "tags": [],
    "kind": "page",
    "status": "published",
    "cover": null,
    "og_image": null,
    "show_cta": null,
    "published_at": 1793975922,
    "updated_at": 1793975922,
    "url": "https://blog.example.com/about/"
  }
}
```
