# Fonts

The font library contains extra fonts for [share images](../platform/share-images.md#fonts).
The files are in the `fonts/` folder of the data folder.
All routes need admin authentication.

## Routes

| Method | Path | Body | Success | Errors |
| --- | --- | --- | --- | --- |
| `GET` | `/api/fonts` | None | `200` array of `FontItem`, ordered by name | `401`, `503` |
| `POST` | `/api/fonts` | Raw font bytes | `200` `FontItem` | `400`, `401`, `403`, `413`, `503` |
| `POST` | `/api/fonts/google` | `GoogleFontInput` | `200` array of `FontItem` | `400`, `401`, `403`, `413`, `415`, `503` |
| `DELETE` | `/api/fonts/{name}` | None | `204` | `400`, `401`, `403`, `404`, `503` |

Each upload, import, and deletion loads the fonts again and clears the page cache of every site.

## Upload

`POST /api/fonts` takes the font file as the raw request body.

- The file must be a TTF, OTF, or TTC font with at least one readable face. Other bytes return `400` with the code `validation` and `field` set to `file`.
- An empty body returns `400` with `field` set to `file`.
- The body can be at most 20 MiB. A larger body returns `413`.
- The server reads the file signature, so the `Content-Type` header is not checked.

The server names the file from the PostScript name of the first face, then the first 8 hexadecimal digits of the SHA-256 hash of the file.
The same file therefore always gets the same name.

## `GoogleFontInput`

The body of `POST /api/fonts/google`.
Unknown fields return `400`.

| Field | Type | Rule |
| --- | --- | --- |
| `url` | [`AbsoluteUrl`](value-types.md#absoluteurl) | A Google Fonts specimen or stylesheet address |

These addresses are accepted:

| Address | Example |
| --- | --- |
| A specimen page | `https://fonts.google.com/specimen/Noto+Sans+JP` |
| A stylesheet | `https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;700` |
| An old stylesheet | `https://fonts.googleapis.com/css?family=Noto+Sans+JP` |

The import works in these steps:

1. The server converts a specimen address to a `css2` stylesheet address.
2. The server downloads the stylesheet from `fonts.googleapis.com`.
3. The server downloads every font file in the stylesheet from `fonts.gstatic.com`. It ignores addresses on other domains.
4. The server checks and stores each file like an upload, with the source `google`.

| Case | Result |
| --- | --- |
| Another domain, path, scheme, or port | `400`, `field` set to `url` |
| A specimen family name with characters other than letters, digits, and spaces | `400`, `field` set to `url` |
| Google Fonts answers with an error, or the stylesheet lists no font file | `400`, `field` set to `url` |
| The stylesheet lists more than 20 files | `400`, `field` set to `url` |
| A file is larger than 20 MiB | `413` |

The server does not follow redirects.
When one file fails, the files that were stored before it stay in the library.

## `FontItem`

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | String | The file name in the font folder |
| `families` | Array of strings | The family names in the file |
| `source` | `"upload"` or `"google"` | The origin of the file |
| `size` | Integer | The file size in bytes |
| `created_at` | Integer | The upload time, Unix seconds |

## Path parameter

`{name}` has ASCII letters, digits, and hyphens, then `.ttf`, `.otf`, or `.ttc`.
Other values return `400` with `field` set to `name`.
An unknown name returns `404`.

## Example

```sh
curl -X POST https://harbor.example.com/api/fonts/google \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://fonts.google.com/specimen/Noto+Sans+JP"}'
```

```json
[
  {
    "name": "NotoSansJP-Regular-0f0f0f0f.ttf",
    "families": ["Noto Sans JP"],
    "source": "google",
    "size": 5732824,
    "created_at": 1794301200
  }
]
```

Upload a local file:

```sh
curl -X POST https://harbor.example.com/api/fonts \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  --data-binary @NotoSansArabic-Regular.ttf
```
