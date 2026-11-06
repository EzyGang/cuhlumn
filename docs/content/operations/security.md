# Security

This page describes the trust model and the protections of the server.

## Trust model

One admin token controls every site on the server.
A token holder can publish raw HTML and scripts on every blog, through post bodies and the `head` setting.
Give the token only to writers and agents that you trust completely.

## Admin token

- The token must have at least 32 bytes.
- The server keeps only the SHA-256 digest of the token in memory.
- The server compares digests in constant time.
- The bearer token works on every domain of the server.

## Console sessions

| Property | Value |
| --- | --- |
| Cookie | `cuhlumn_session` |
| Attributes | `HttpOnly`, `SameSite=Strict`, `Path=/`, `Max-Age=604800`, and `Secure` when `CUHLUMN_SCHEME=https` |
| Session identifier | 32 random bytes |
| Stored value | The SHA-256 hash of the token digest and the identifier |
| Lifetime | 7 days from login, not extended by use |

- The server accepts the cookie only on the console domain.
- Because the stored value includes the token digest, a new token ends all sessions.
- The server deletes expired sessions at startup and at each login.

## Cross-site requests

A cookie request that changes data must have an `Origin` header for the console domain.
Otherwise the server returns `403`.
`SameSite=Strict` gives a second protection.

## Console page

The console uses a Content Security Policy that allows scripts only from the console domain and blocks framing.
The editor preview runs in a sandboxed frame without scripts.

## Uploads

- The server accepts only PNG, JPEG, WebP, GIF, and AVIF, and checks the first bytes of each file.
- File names come from the content hash, so a request cannot choose a path.
- Media responses have `X-Content-Type-Options: nosniff`.

## Errors

Internal errors return a fixed message.
The cause goes only to the server log.

## Network

The server listens on plain HTTP.
Put a TLS proxy in front of it, and set `CUHLUMN_SCHEME=https` in production.
