# Analytics

The analytics route returns the page views, the daily visitors, and the link clicks of one site.
The site is a blog or an [external site](../platform/analytics.md#external-sites).
The route needs admin authentication.
See [Analytics](../platform/analytics.md) for what the server counts.

## Routes

| Method | Path | Query | Success | Errors |
| --- | --- | --- | --- | --- |
| `GET` | `/api/sites/{host}/analytics` | `period` | `200` `SiteAnalytics` | `400`, `401`, `404`, `503` |

Before the server reads the report, it writes the counts that it holds in memory, so the report is current.

## Query

| Parameter | Values | Default |
| --- | --- | --- |
| `period` | `week` (7 days), `month` (30 days), `quarter` (90 days), `year` (365 days) | `month` |

The period ends today (UTC) and includes today.
Another value or another parameter returns `400` with the code `validation` and `field` set to `period`.

## `SiteAnalytics`

| Field | Type | Meaning |
| --- | --- | --- |
| `period` | String | The period of the report |
| `from` | String | The first day, `YYYY-MM-DD` (UTC) |
| `to` | String | The last day, `YYYY-MM-DD` (UTC) |
| `views` | Integer | The page views in the period |
| `visitors` | Integer | The sum of the daily visitors in the period |
| `days` | Array of `DayCount` | One entry for each day, oldest first, with `0` for a day without views |
| `pages` | Array of `PageCount` | The 10 pages with the most views |
| `referrers` | Array of `ReferrerCount` | The 10 referrers with the most views |
| `devices` | Array of `DeviceCount` | The views of each device class, most views first |
| `browsers` | Array of `BrowserCount` | The views of each browser family, most views first |
| `outbound` | Array of `ClickCount` | The 10 outbound link targets with the most clicks |
| `downloads` | Array of `ClickCount` | The 10 download targets with the most clicks |

| Object | Fields |
| --- | --- |
| `DayCount` | `date` (`YYYY-MM-DD`), `views`, `visitors` |
| `PageCount` | `path`, `title` (the post title, the site title for `/`, or `null`), `views` |
| `ReferrerCount` | `host` (the referring domain, or `null` for direct visits), `views` |
| `DeviceCount` | `device` (`desktop`, `mobile`, or `tablet`), `views` |
| `BrowserCount` | `browser` (`chrome`, `safari`, `firefox`, `edge`, `opera`, `samsung`, or `other`), `views` |
| `ClickCount` | `target` (the host name and the path of the link, such as `github.com/EzyGang/cuhlumn`), `clicks` |

The `title` of a page is `null` for a tag page, for a page that no longer exists, and for every page of an external site.

## Public beacon route

`POST /_hit` records one page view or one link click.
The page script sends it, and it needs no authentication.
The main domain of every blog accepts the route.

| Body | Meaning |
| --- | --- |
| `{"path": "...", "referrer": "..."}` | A view of a blog page. The blog is the domain of the request |
| `{"click": "...", "target": "..."}` | A link click on a blog page. The blog is the domain of the request |
| `{"host": "...", "path": "...", "referrer": "..."}` | A view of a page of the external site `host` |
| `{"host": "...", "click": "...", "target": "..."}` | A link click on a page of the external site `host` |

| Field | Value |
| --- | --- |
| `host` | The host name of the page, from `location.hostname` |
| `path` | The path of the page, from `location.pathname` |
| `referrer` | The referring address, from `document.referrer`, or an empty string |
| `click` | `outbound` or `download` |
| `target` | The host name and the path of the link, without a scheme, for example `github.com/EzyGang/cuhlumn` |

| Case | Result |
| --- | --- |
| One of these JSON bodies, of at most 2 KiB | `204`, with `Cache-Control: no-store` |
| A body that is larger than 2 KiB | `413` |
| A body that is not one of these JSON objects, or that has other fields | `400` |

The server answers `204` also when it ignores the request.
It ignores a request in these cases:

- The `User-Agent` header is empty or looks like a bot.
- The body has no `host`, and the domain of the request is not the main domain of a blog.
- The body is a view without a `host`, and `path` is not a page of the blog.
- The body has a `host` that is not a registered external site.
- The body has a `host`, and the `Origin` header does not name the same domain.
- The `path` or the `target` breaks a [limit](../platform/analytics.md#limits-for-external-sites).

The server removes the query and the fragment from the `path` of an external site and from each `target`.
It also decodes their percent-encoding.
Slugs cannot contain `_`, so the route never collides with a post.

## Public script route

`GET /_hit.js` on the main domain of a blog returns the page script for external sites.
It needs no authentication.

| Property | Value |
| --- | --- |
| `Content-Type` | `text/javascript; charset=utf-8` |
| `Cache-Control` | `public, max-age=86400` |
| `ETag` | A hash of the file. A request with a matching `If-None-Match` gets `304` |
| Size | Less than 1 KB |

The file is the same script that every blog page contains.
An alias domain answers with a redirect to the main domain.
See [Add the script](../platform/analytics.md#add-the-script) for the tag.

## Example

```sh
curl "https://harbor.example.com/api/sites/harbor.example.com/analytics?period=week" \
  -H "Authorization: Bearer $CUHLUMN_TOKEN"
```

```json
{
  "period": "week",
  "from": "2026-11-04",
  "to": "2026-11-10",
  "views": 412,
  "visitors": 268,
  "days": [
    { "date": "2026-11-04", "views": 51, "visitors": 33 },
    { "date": "2026-11-05", "views": 64, "visitors": 41 }
  ],
  "pages": [
    { "path": "/sqlite-backups/", "title": "How we back up SQLite", "views": 230 },
    { "path": "/", "title": "Harbor Notes", "views": 121 }
  ],
  "referrers": [
    { "host": null, "views": 198 },
    { "host": "news.ycombinator.com", "views": 160 }
  ],
  "devices": [
    { "device": "desktop", "views": 301 },
    { "device": "mobile", "views": 111 }
  ],
  "browsers": [
    { "browser": "chrome", "views": 240 },
    { "browser": "firefox", "views": 98 },
    { "browser": "safari", "views": 74 }
  ],
  "outbound": [
    { "target": "github.com/EzyGang/cuhlumn", "clicks": 57 },
    { "target": "harbor.example.com/", "clicks": 31 }
  ],
  "downloads": [
    { "target": "harbor.example.com/files/harbor-cli.tar.gz", "clicks": 12 }
  ]
}
```

The example shortens the `days` array.
