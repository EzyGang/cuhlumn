# HTTP API overview

The HTTP API manages sites, posts, media, fonts, server settings, and console sessions.
Every route is under `/api` and answers on every domain of the server.

With `CUHLUMN_ADMIN_ADDR`, the API is only on the admin port.
The public port then answers only `GET /api/health`, and every other `/api` path returns `404` with the code `not_found`.
See [Admin listener](../operations/configuration.md#admin-listener).

## Authentication

Admin routes accept one of these credentials:

- `Authorization: Bearer <token>` on any domain
- The `cuhlumn_session` cookie, only on a domain that serves the console, or on every host of the admin port

The bearer token has precedence.
A wrong bearer token returns `401`, even when a valid cookie is present.
An `Authorization` header with another scheme, such as `Basic` from a proxy, is ignored, and the cookie decides.

Cookie requests with a method that changes data (`POST`, `PUT`, `DELETE`) must send an `Origin` header for the console domain of the request.
Browsers send this header automatically.
On the admin port, the `Origin` host and port must equal the `Host` header.

The admin token comes from `CUHLUMN_TOKEN`, or from the database after the first setup.
Before the first setup, every admin route and every session route returns `503` with the code `needs_setup`.
`GET /api/health` and `POST /api/setup` work before the setup.
See [Security](../operations/security.md) for the token and session rules.

## Routes

| Method | Path | Authentication | Body | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/health` | None | None | `200` [`Health`](#health) |
| `POST` | `/api/setup` | None, only before the first setup | [`TokenInput`](settings.md#tokeninput) | `200` `Session`, with `Set-Cookie` on the console domain |
| `GET` | `/api/settings` | Admin | None | `200` [`ServerSettings`](settings.md#serversettings) |
| `PUT` | `/api/settings` | Admin | [`ServerSettingsInput`](settings.md#serversettingsinput) | `200` `ServerSettings` |
| `PUT` | `/api/settings/token` | Admin | `TokenInput` | `200` `Session`, with `Set-Cookie` on the console domain |
| `GET` | `/api/fonts` | Admin | None | `200` array of [`FontItem`](fonts.md#fontitem) |
| `POST` | `/api/fonts` | Admin | Raw font bytes | `200` `FontItem` |
| `POST` | `/api/fonts/google` | Admin | [`GoogleFontInput`](fonts.md#googlefontinput) | `200` array of `FontItem` |
| `DELETE` | `/api/fonts/{name}` | Admin | None | `204` |
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
| `GET` | `/api/sites/{host}/analytics` | Admin | None | `200` [`SiteAnalytics`](analytics.md#siteanalytics) |

An unknown `/api` path returns `404`.
A known path with a wrong method returns `405`.
Both answers use the [error format](errors.md).

Outside `/api`, every blog host, console host, and the admin listener serve `GET /_fonts/{name}` without authentication.
See [Public font route](fonts.md#public-font-route).
Every blog host also accepts `POST /_hit` without authentication.
See [Public beacon route](analytics.md#public-beacon-route).

## Status codes

| Status | Routes | Cause |
| --- | --- | --- |
| `400` | All routes with a body or a path parameter | Invalid JSON or a value that breaks a rule, such as a `brand.font_family` that no library font has |
| `401` | Admin routes, `POST /api/session` | A missing or wrong token or session |
| `403` | Cookie requests that change data, `POST /api/session`, `POST /api/setup` | The `Origin` header names another domain |
| `404` | Routes with a path parameter, session routes | The object does not exist, or the domain does not serve the console |
| `409` | `POST /api/setup`, `PUT /api/settings`, `PUT /api/settings/token`, site and post writes | See [Conflicts](errors.md#conflicts) and [Locked values](errors.md#locked-values) |
| `413` | Routes with a body | The body is larger than the limit |
| `415` | JSON routes, media upload | The content type is not accepted |
| `503` | Admin routes and session routes | The server has no admin token yet (`needs_setup`) |

## Path parameters

- `{host}` is the canonical domain of a site. An alias returns `404`. The server lowercases the value and removes trailing dots.
- `{slug}` follows the [slug rules](value-types.md#slug).
- `{name}` is a media file name such as `ba7816bf8f01cfea414140de5dae2223.png`, or a font file name such as `Inter-Regular-1a2b3c4d.ttf`.

An invalid path parameter returns `400` with the code `validation` and the parameter name in `field`.

## Request bodies

- JSON routes require `Content-Type: application/json` or a media type that ends in `+json`. Other types return `415`.
- JSON input objects reject unknown fields.
- Omitted optional fields take their default values.
- A `PUT` replaces the complete object.
- JSON bodies can be at most 2 MB. Media uploads can be at most 16 MiB. Font uploads can be at most 20 MiB. A larger body returns `413`.

## Timestamps

All timestamps are Unix time in seconds.

## Health

`GET /api/health` needs no authentication and works before the setup.

```json
{ "ok": true, "version": "0.18.0", "needs_setup": false }
```

`needs_setup` is `true` while the server has no admin token.
The console then shows the **Setup** page.

## Example

```sh
curl https://blog.example.com/api/sites \
  -H "Authorization: Bearer $CUHLUMN_TOKEN"
```
