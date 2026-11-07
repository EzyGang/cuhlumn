# Security

This page describes the trust model and the protections of the server.

## Trust model

One admin token controls every site on the server.
A token holder can publish raw HTML and scripts on every blog, through post bodies and the `head` setting.
Give the token only to writers and agents that you trust completely.

## First launch

Without an admin token, the server starts in setup mode, and the first visitor of the console sets the token.

!!! warning "Anyone who reaches the server first can claim it"
    Before the setup, the console answers on every domain that is not a blog domain, also on the IP address of the server.
    A person who finds the server before you can set the token and control every site.
    To prevent this, do one of these steps:

    - Set `CUHLUMN_TOKEN` before the first start.
    - Keep the port private until you complete the setup.

Only the first `POST /api/setup` request succeeds.
Later requests return `409` with the code `conflict`.

## Admin token

- The token must have at least 32 bytes.
- The database stores only the SHA-256 digest of the token.
- The server compares digests in constant time.
- The bearer token works on every domain of the server.
- When `CUHLUMN_TOKEN` is set, it overrides the token in the database, and the console cannot change it.
- A new token, from the console or from `CUHLUMN_TOKEN`, ends all sessions.

## Console sessions

| Property | Value |
| --- | --- |
| Cookie | `cuhlumn_session` |
| Attributes | `HttpOnly`, `SameSite=Strict`, `Path=/`, `Max-Age=604800`, and `Secure` when the scheme is `https` |
| Session identifier | 32 random bytes |
| Stored value | The SHA-256 hash of the token digest and the identifier |
| Lifetime | 7 days from login, not extended by use |

- The server accepts the cookie only on the console domain.
- Without a saved console domain, every domain that is not a blog domain counts as the console domain.
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
Put a TLS proxy in front of it, and keep the scheme at `https` in production.
The Compose file publishes the port only on `127.0.0.1`.
