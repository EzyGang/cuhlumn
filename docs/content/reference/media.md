# Media

## Routes

| Method | Path | Success | Errors |
| --- | --- | --- | --- |
| `POST` | `/api/sites/{host}/media` | `200` `MediaSaved` | `400`, `404`, `413`, `415` |
| `GET` | `/api/sites/{host}/media` | `200` array of `MediaItem`, newest first | `400`, `404` |
| `DELETE` | `/api/sites/{host}/media/{name}` | `204` | `400`, `404` |

## Upload

The body is the raw image.
The `Content-Type` header must be one of these types, without regard to case and parameters:

| Content-Type | Extension | First bytes |
| --- | --- | --- |
| `image/png` | `png` | `89 50 4E 47 0D 0A 1A 0A` |
| `image/jpeg` | `jpg` | `FF D8 FF` |
| `image/webp` | `webp` | `RIFF` and `WEBP` at byte 8 |
| `image/gif` | `gif` | `GIF87a` or `GIF89a` |
| `image/avif` | `avif` | An `ftyp` box with the brand `avif` or `avis` |

| Case | Result |
| --- | --- |
| Body larger than 16 MiB | `413` |
| Other `Content-Type` | `415` |
| Empty body | `400` |
| First bytes do not match the type | `415` |
| Unknown site | `404` |
| Same image again | `200` with the same name; the first upload time stays |

## `MediaSaved`

| Field | Type | Example |
| --- | --- | --- |
| `name` | String | `ba7816bf8f01cfea414140de5dae2223.png` |
| `url` | String | `https://blog.example.com/media/ba7816bf8f01cfea414140de5dae2223.png` |

## `MediaItem`

| Field | Type |
| --- | --- |
| `name` | String |
| `url` | String |
| `content_type` | String |
| `size` | Integer, bytes |
| `created_at` | Integer, Unix seconds |

## Names

A name is 32 lowercase hexadecimal characters, a dot, and one of the extensions above.
Another value in `{name}` returns `400` with the field `name`.

## Storage

The server writes each file to `{CUHLUMN_DATA}/media/{host}/{name}` through a temporary file and a rename.
Deleting a site removes its media folder.
