# Sessions

The session routes support the management console.
They answer only on a domain that serves the console.

- With a console domain, only that domain serves the console.
- Without a console domain, every domain that is not a site domain or an alias serves the console.

On other domains, the session routes return `404`.
Before the first setup, they return `503` with the code `needs_setup`.

## Routes

| Method | Path | Success | Errors |
| --- | --- | --- | --- |
| `POST` | `/api/session` | `200` `Session` and `Set-Cookie` | `401`, `403`, `404`, `503` |
| `GET` | `/api/session` | `200` `Session` | `401`, `404`, `503` |
| `DELETE` | `/api/session` | `204`, the cookie is cleared | `403`, `404`, `503` |

## `SessionInput`

| Field | Type |
| --- | --- |
| `token` | String, the admin token |

## `Session`

| Field | Type |
| --- | --- |
| `expires_at` | Integer, Unix seconds |

## Rules

- `POST` checks the token in constant time. A wrong token returns `401`.
- `POST` with an `Origin` header for another domain returns `403`.
- `GET` reads only the cookie. It does not extend the session.
- `DELETE` with a cookie requires an `Origin` header for the console domain.
- A session lasts 7 days.

## Sessions from settings routes

Two other routes also start a session and return `Session`:

| Route | Effect |
| --- | --- |
| `POST /api/setup` | Stores the first admin token and starts a session |
| `PUT /api/settings/token` | Deletes every session, stores the new token, and starts a new session |

On a console domain, both routes also set the session cookie.
On other domains, they return `Session` without a cookie.
See [Server settings](settings.md).

The server derives each session hash from the token digest.
A new token from `CUHLUMN_TOKEN` therefore also ends every session after a restart.

See [Security](../operations/security.md#console-sessions) for the cookie attributes.
