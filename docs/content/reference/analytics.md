# Analytics

The analytics route returns the page views and the daily visitors of one site.
It needs admin authentication.
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

| Object | Fields |
| --- | --- |
| `DayCount` | `date` (`YYYY-MM-DD`), `views`, `visitors` |
| `PageCount` | `path`, `title` (the post title, the site title for `/`, or `null` for a tag page or a page that no longer exists), `views` |
| `ReferrerCount` | `host` (the referring domain, or `null` for direct visits), `views` |
| `DeviceCount` | `device` (`desktop`, `mobile`, or `tablet`), `views` |
| `BrowserCount` | `browser` (`chrome`, `safari`, `firefox`, `edge`, `opera`, `samsung`, or `other`), `views` |

## Public beacon route

`POST /_hit` on a blog domain records one view.
The inline script of each HTML page sends it, and it needs no authentication.

| Case | Result |
| --- | --- |
| A JSON body `{"path": "...", "referrer": "..."}` of at most 2 KiB | `204`, with `Cache-Control: no-store` |
| A body that is larger than 2 KiB | `413` |
| A body that is not this JSON object, or that has other fields | `400` |

The server answers `204` also when it ignores the view, for example for a bot or an unknown path.
Slugs cannot contain `_`, so the route never collides with a post.

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
  ]
}
```

The example shortens the `days` array.
