# Announcements

A site can show one announcement bar above every HTML page.
Use it for news such as a release, a maintenance window, or a new post.
The `announcement` field of the site contains the bar, or `null` for no bar.
The console shows it in the **Announcement** section of the site settings.

## Fields

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `text` | String | Required | Plain text, trimmed, 1 to 160 characters, no control characters |
| `link` | [`LinkUrl`](../reference/value-types.md#linkurl) or `null` | `null` | |
| `link_label` | String | `"Learn more"` | Trimmed, at most 40 characters, no control characters |
| `tone` | `info`, `accent`, or `warning` | `info` | |
| `starts_at` | Integer or `null` | `null` | Unix seconds |
| `ends_at` | Integer or `null` | `null` | Unix seconds, after `starts_at` when both are set |
| `dismissible` | Boolean | `true` | Lets readers close the bar |

An empty `link_label` becomes `"Learn more"`.
The page shows the link only when `link` is set.

## Schedule

The bar is live when `starts_at ≤ now < ends_at`.
A missing bound is open, so a bar without both times is always live.

| Status | Condition |
| --- | --- |
| Scheduled | `starts_at` is in the future |
| Live | The bar shows on the pages |
| Ended | `ends_at` is now or in the past |

The site stores an ended announcement until you clear or replace it.
The console shows the status **Off** when the site has no announcement.

## Tones

| `tone` | Use |
| --- | --- |
| `info` | Neutral news |
| `accent` | News in the accent color of the site |
| `warning` | A problem or a maintenance window |

## HTML

The bar is the first element in `<body>`:

```html
<div class="announcement announcement-warning" role="region" aria-label="Announcement" data-announcement="4391f29f">
  <p class="announcement-text">Backups moved <a href="/sqlite-backups/">Learn more<span class="announcement-context"> about this announcement</span></a></p>
  <button type="button" class="announcement-close" aria-label="Dismiss the announcement">…</button>
</div>
```

The close button is present only when `dismissible` is `true`.
The link contains the hidden text "about this announcement", so screen readers and search engines get a descriptive link name.

## Dismissal

The `data-announcement` key is the first 8 hexadecimal digits of the SHA-256 hash of the text, a line feed, and the link.

1. The reader selects the close button.
2. The browser stores `announcement:{key}` in `localStorage` and hides the bar.
3. On the next page, a script in `<head>` hides the bar before the first paint.

A new text or a new link gives a new key, so readers see the changed bar again.
A change of the label, the tone, or the times keeps the key.
A bar that is not dismissible has no scripts.

## Where the bar does not appear

- The Atom feed, the sitemap, `robots.txt`, `llms.txt`, and `llms-full.txt`
- The Markdown copies (`/{slug}.md`)
- The share images

## Caching

A cached HTML page expires at the next start or end time of the announcement.
The next request after that time renders the page again, with or without the bar.
Browsers and CDNs can show the old page for up to the `max-age` of the page, which is 60 seconds.
See [Caching](caching.md).

## Example

The CLI command `cuhlumn announce` sets the bar. See [Command-line client](../cli/index.md).
Through the API, send the `announcement` field in the site body:

```json
{
  "title": "Harbor Notes",
  "announcement": {
    "text": "Backups moved",
    "link": "/sqlite-backups/",
    "tone": "warning",
    "starts_at": 1794301200,
    "ends_at": 1794765600
  }
}
```
