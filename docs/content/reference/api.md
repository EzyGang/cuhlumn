# HTTP API overview

The HTTP API manages sites, posts, media, and console sessions.
Every route is under `/api` and answers on every domain of the server.

## Authentication

Admin routes accept one of these credentials:

- `Authorization: Bearer <CUHLUMN_TOKEN>` on any domain
- The `cuhlumn_session` cookie, only on the console domain

The bearer token has precedence.
A wrong bearer token returns `401`, even when a valid cookie is present.
An `Authorization` header with another scheme, such as `Basic` from a proxy, is ignored, and the cookie decides.

Cookie requests with a method that changes data (`POST`, `PUT`, `DELETE`) must send an `Origin` header for the console domain.
Browsers send this header automatically.

Without `CUHLUMN_TOKEN`, every admin route and every session route returns `503` with the code `api_disabled`.
See [Security](../operations/security.md) for the token and session rules.

## Routes

| Method | Path | Authentication | Body | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/health` | None | None | `200` [`Health`](#health) |
| `POST` | `/api/session` | Console domain | [`SessionInput`](sessions.md) | `200` `Session` and `Set-Cookie` |
| `GET` | `/api/session` | Console cookie | None | `200` `Session` |
| `DELETE` | `/api/session` | Console cookie | None | `204` |
| `GET` | `/api/sites` | Admin | None | `200` array of [`Site`](sites.md) |
| `GET` | `/api/sites/{host}` | Admin | None | `200` `Site` |
| `PUT` | `/api/sites/{host}` | Admin | [`SiteInput`](sites.md#siteinput) | `200` `Site` |
| `DELETE` | `/api/sites/{host}` | Admin | None | `204` |
| `GET` | `/api/sites/{host}/posts` | Admin | None | `200` array of [`Post`](posts.md) |
| `GET` | `/api/sites/{host}/posts/{slug}` | Admin | None | `200` `Post` |
| `PUT` | `/api/sites/{host}/posts/{slug}` | Admin | [`PostInput`](posts.md#postinput) | `200` `PostSaved` |
| `DELETE` | `/api/sites/{host}/posts/{slug}` | Admin | None | `204` |
| `POST` | `/api/sites/{host}/media` | Admin | Raw image bytes | `200` [`MediaSaved`](media.md) |
| `GET` | `/api/sites/{host}/media` | Admin | None | `200` array of `MediaItem` |
| `DELETE` | `/api/sites/{host}/media/{name}` | Admin | None | `204` |
| `POST` | `/api/sites/{host}/preview` | Admin | [`PreviewInput`](preview.md) | `200` `Preview` |

An unknown `/api` path returns `404`.
A known path with a wrong method returns `405`.
Both answers use the [error format](errors.md).

## Path parameters

- `{host}` is the canonical domain of a site. An alias returns `404`. The server lowercases the value and removes trailing dots.
- `{slug}` follows the [slug rules](value-types.md#slug).
- `{name}` is a media file name such as `ba7816bf8f01cfea414140de5dae2223.png`.

An invalid path parameter returns `400` with the code `validation` and the parameter name in `field`.

## Request bodies

- JSON routes require `Content-Type: application/json` or a media type that ends in `+json`. Other types return `415`.
- JSON input objects reject unknown fields.
- Omitted optional fields take their default values.
- A `PUT` replaces the complete object.
- JSON bodies can be at most 2 MB. Media uploads can be at most 16 MiB. A larger body returns `413`.

## Timestamps

All timestamps are Unix time in seconds.

## Health

`GET /api/health` needs no authentication and works without a token.

```json
{ "ok": true, "version": "0.1.0" }
```

## Example

```sh
curl https://blog.example.com/api/sites \
  -H "Authorization: Bearer $CUHLUMN_TOKEN"
```
