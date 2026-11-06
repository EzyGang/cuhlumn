# Preview

`POST /api/sites/{host}/preview` renders Markdown with the same renderer as the public pages.
The answer contains the body HTML only, without the page layout or CSS.

## `PreviewInput`

| Field | Type | Rule |
| --- | --- | --- |
| `body` | String | Required |

## `Preview`

| Field | Type | Meaning |
| --- | --- | --- |
| `html` | String | The rendered body |
| `toc` | Array of `TocEntry` | Every second- and third-level heading |
| `word_count` | Integer | The number of words, code included |

The public page shows the table of contents only when `toc` has three or more entries.

## `TocEntry`

| Field | Type |
| --- | --- |
| `level` | Integer, `2` or `3` |
| `id` | String, the heading id |
| `label` | String, the heading text |

## Errors

| Case | Result |
| --- | --- |
| Unknown site | `404` |
| Body is not JSON | `400` or `415` |

A preview does not change stored data or the page cache.

## Example

```sh
curl -X POST https://blog.example.com/api/sites/blog.example.com/preview \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"body": "## Why one file\n\nIt is easy to copy."}'
```

```json
{
  "html": "<h2 id=\"why-one-file\">Why one file<a class=\"heading-link\" href=\"#why-one-file\" aria-hidden=\"true\" tabindex=\"-1\">#</a></h2>\n<p>It is easy to copy.</p>\n",
  "toc": [{ "level": 2, "id": "why-one-file", "label": "Why one file" }],
  "word_count": 4
}
```
