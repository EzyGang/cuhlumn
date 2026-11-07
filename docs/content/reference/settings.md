# Server settings

The server settings are the admin token, the console domain, and the scheme of public addresses.
Environment variables can set each of them.
A value from the environment wins over the database value and locks the field.

| Field | Variable | Default |
| --- | --- | --- |
| Admin token | `CUHLUMN_TOKEN` | None, so the server needs the setup |
| `console_host` | `CUHLUMN_CONSOLE_HOST` | None, so every domain that is not a site serves the console |
| `scheme` | `CUHLUMN_SCHEME` | `https` |

## Routes

| Method | Path | Body | Success | Errors |
| --- | --- | --- | --- | --- |
| `POST` | `/api/setup` | `TokenInput` | `200` `Session` | `400`, `403`, `409`, `415` |
| `GET` | `/api/settings` | None | `200` `ServerSettings` | `401`, `503` |
| `PUT` | `/api/settings` | `ServerSettingsInput` | `200` `ServerSettings` | `400`, `401`, `403`, `409`, `415`, `503` |
| `PUT` | `/api/settings/token` | `TokenInput` | `200` `Session` | `400`, `401`, `403`, `409`, `415`, `503` |

`GET` and `PUT` routes need admin authentication.
`POST /api/setup` needs no authentication.

## `TokenInput`

The body of `POST /api/setup` and `PUT /api/settings/token`.
Unknown fields return `400`.

| Field | Type | Rule |
| --- | --- | --- |
| `token` | String | 32 to 512 bytes |

## `ServerSettingsInput`

The body of `PUT /api/settings`.
Unknown fields return `400`.

| Field | Type | Default |
| --- | --- | --- |
| `console_host` | [`Host`](value-types.md#host) or `null` | `null` |
| `scheme` | `"http"` or `"https"` | `"https"` |

## `ServerSettings`

The answer of `GET /api/settings` and `PUT /api/settings`.
It contains the effective values, so a locked field shows the value from the environment.

| Field | Type | Meaning |
| --- | --- | --- |
| `console_host` | `Host` or `null` | The only domain that serves the console |
| `scheme` | `"http"` or `"https"` | The scheme of absolute addresses and of the cookie `Secure` flag |
| `locked` | `Locked` | The fields that environment variables set |

## `Locked`

| Field | Type | Meaning |
| --- | --- | --- |
| `token` | Boolean | `CUHLUMN_TOKEN` is set |
| `console_host` | Boolean | `CUHLUMN_CONSOLE_HOST` is set |
| `scheme` | Boolean | `CUHLUMN_SCHEME` is set |

## First setup

`POST /api/setup` stores the first admin token.

- It works only while the server has no token in the environment or the database.
- After that, it returns `409` with the code `conflict`.
- When two requests arrive at the same time, the first one wins and the other gets `409`.
- The database stores only the SHA-256 digest of the token.
- The answer starts a session. On a console domain, it also sets the session cookie.
- On a console domain, a request with an `Origin` header for another domain returns `403`.

## Settings update

`PUT /api/settings` stores `console_host` and `scheme`.

| Case | Result |
| --- | --- |
| A locked field with a value that is not the effective value | `409` with the code `locked` and the field name in `field` |
| A locked field with the effective value | The server keeps the locked value |
| `console_host` is a site domain or an alias | `409` with the code `conflict` and `field` set to `console_host` |
| `scheme` changes | The server clears the page cache of every site |

A new console domain takes effect immediately.
The session cookie belongs to the old domain, so log in again on the new domain.

## Token change

`PUT /api/settings/token` replaces the admin token.

1. The server stores the digest of the new token.
2. The server deletes every session.
3. The answer starts a new session. On a console domain, it also sets the new session cookie.

The CLI, scripts, and agents need the new token.
When `CUHLUMN_TOKEN` is set, the route returns `409` with the code `locked` and `field` set to `token`.

## Examples

Set the first token:

```sh
curl -X POST https://console.harbor.example.com/api/setup \
  -H "Content-Type: application/json" \
  -d '{"token": "harbor-token-0123456789-abcdefghij"}'
```

```json
{ "expires_at": 1795000000 }
```

Move the console to its own domain:

```sh
curl -X PUT https://harbor.example.com/api/settings \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"console_host": "console.harbor.example.com", "scheme": "https"}'
```

```json
{
  "console_host": "console.harbor.example.com",
  "scheme": "https",
  "locked": { "token": false, "console_host": false, "scheme": false }
}
```
