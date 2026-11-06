# Sessions

The session routes support the management console.
They answer only on the console domain.
On other domains, and when `CUHLUMN_CONSOLE_HOST` is not set, they return `404`.

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

See [Security](../operations/security.md#console-sessions) for the cookie attributes.
