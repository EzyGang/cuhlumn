# Sites

## Routes

| Method | Path | Success | Errors |
| --- | --- | --- | --- |
| `GET` | `/api/sites` | `200` array of `Site`, ordered by domain | `401`, `503` |
| `GET` | `/api/sites/{host}` | `200` `Site` | `400`, `404` |
| `PUT` | `/api/sites/{host}` | `200` `Site` | `400`, `409`, `413`, `415` |
| `DELETE` | `/api/sites/{host}` | `204` | `400`, `404` |

`PUT` creates the site or replaces all its settings.
An omitted field takes its default value, except `indexnow_key`, which keeps the stored key.

## `SiteInput`

The body of `PUT /api/sites/{host}`.
Unknown fields return `400`.

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `title` | String | Required | Trimmed, not blank, at most 200 characters |
| `description` | String | `""` | No control characters |
| `aliases` | Array of [`Host`](value-types.md#host) | `[]` | Repeated values and the site's own domain are removed |
| `author` | String | `""` | No control characters |
| `author_url` | [`AbsoluteUrl`](value-types.md#absoluteurl) or `null` | `null` | |
| `project_name` | String | `""` | No control characters |
| `project_url` | `AbsoluteUrl` or `null` | `null` | |
| `cta` | String | `""` | No control characters |
| `lang` | [`Lang`](value-types.md#lang) | `"en"` | |
| `home` | [`Slug`](value-types.md#slug) or `null` | `null` | Must name an existing page of the site |
| `indexnow_key` | [`IndexNowKey`](value-types.md#indexnowkey) or `null` | `null` | `null` keeps the stored key |
| `brand` | [`Brand`](#brand) | Defaults below | |

## `Brand`

All fields are optional.
Unknown fields return `400`.

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `accent` | [`Accent`](value-types.md#accent) | `"#2563eb"` | |
| `font` | `"serif"` or `"sans"` | `"serif"` | |
| `logo` | [`LinkUrl`](value-types.md#linkurl) or `null` | `null` | |
| `favicon` | `LinkUrl` or `null` | `null` | |
| `og_image` | `LinkUrl` or `null` | `null` | |
| `links` | Array of [`Link`](#link) | `[]` | At most 8 |
| `head` | String | `""` | At most 20,000 bytes |
| `footer` | String | `""` | At most 500 characters, no control characters |
| `credit` | Boolean | `true` | |

## `Link`

| Field | Type | Rule |
| --- | --- | --- |
| `label` | String | Trimmed, 1 to 40 characters, no control characters |
| `url` | `LinkUrl` | |

## `Site`

The answer of the site routes.

| Field | Type |
| --- | --- |
| `host` | `Host` |
| `title` | String |
| `description` | String |
| `aliases` | Array of `Host`, sorted |
| `author` | String |
| `author_url` | `AbsoluteUrl` or `null` |
| `project_name` | String |
| `project_url` | `AbsoluteUrl` or `null` |
| `cta` | String |
| `lang` | `Lang` |
| `home` | `Slug` or `null` |
| `indexnow_key` | `IndexNowKey` |
| `brand` | `Brand` |
| `updated_at` | Integer, Unix seconds |

## Rules that use stored data

| Case | Result |
| --- | --- |
| `{host}` is an alias of another site | `409` |
| An alias belongs to another site | `409` |
| `home` names no entry of the site | `400`, field `home` |
| `home` names a post, not a page | `400`, field `home` |
| `{host}` is an alias in `GET` or `DELETE` | `404` |

A new site cannot set `home` in its first `PUT`, because it has no pages yet.
Create the page first, then set `home`.

## Example

```sh
curl -X PUT https://blog.example.com/api/sites/blog.example.com \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Harbor Notes",
    "aliases": ["www.blog.example.com"],
    "brand": {
      "accent": "#0e7490",
      "font": "sans",
      "logo": "/media/ba7816bf8f01cfea414140de5dae2223.png",
      "links": [{ "label": "Status", "url": "https://status.harbor.example.com" }],
      "footer": "© Example Inc.",
      "credit": false
    }
  }'
```
