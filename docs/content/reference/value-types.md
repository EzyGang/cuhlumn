# Value types

The API checks these values in every request and every path parameter.
An invalid value returns `400` with the code `validation`.

## Text

- "Characters" means Unicode scalar values. "Bytes" means UTF-8 bytes.
- Text fields reject control characters, except tab, line feed, and carriage return.
- The `head` setting is raw HTML and is not checked for control characters.

## Host

- Not empty, at most 253 bytes
- Labels separated by dots, each 1 to 63 characters of `a-z`, `0-9`, and `-`
- A label cannot start or end with `-`
- Single-label names such as `localhost` are allowed
- Ports and `_` are not allowed

The server lowercases the value and removes trailing dots.

## Slug

- 1 to 100 bytes
- Lowercase letters and digits, with single hyphens between groups: `[a-z0-9]+(-[a-z0-9]+)*`
- Reserved: `api`, `og`, `media`, `tags`

## Accent

`#rgb` or `#rrggbb` with hexadecimal digits.
The server stores the value in lowercase.
The default is `#2563eb`.

## Lang

2 to 15 bytes of ASCII letters, digits, and `-`, starting with a letter.
The default is `en`.

## AbsoluteUrl

- Starts with `https://` or `http://`
- Has a domain part
- At most 2048 bytes
- No spaces and none of `"`, `'`, `<`, `>`, or `\`

## LinkUrl

An `AbsoluteUrl`, or a root-relative path that starts with one `/`, such as `/media/logo.png`.
A path that starts with `//` is not allowed.
The same character rules apply.

## IndexNowKey

8 to 128 bytes of ASCII letters, digits, and `-`.

## Tag

Trimmed, not blank, at most 50 characters, no control characters.

## Tags

The server removes repeated tags without regard to case, keeps the first spelling, and keeps the order.
After that, a post can have at most 20 tags.

## Enumerations

| Type | Values | Default |
| --- | --- | --- |
| `Kind` | `post`, `page` | `post` |
| `Status` | `draft`, `published` | `draft` |
| `Font` | `serif`, `sans` | `serif` |
| `Palette` | `paper`, `slate`, `sepia`, `mono`, `contrast` | `paper` |
| `Theme` | `system`, `light`, `dark` | `system` |
| `Width` | `narrow`, `normal`, `wide` | `normal` |
| `HomeLayout` | `list`, `compact`, `cards` | `list` |
| `CodeTheme` | `github`, `solarized`, `ocean`, `none` | `github` |
| `CardStyle` | `dark`, `light`, `accent` | `dark` |
| `Network` | `github`, `gitlab`, `codeberg`, `mastodon`, `bluesky`, `x`, `threads`, `linkedin`, `youtube`, `instagram`, `discord`, `telegram`, `email` | None, required |
| `Tone` | `info`, `accent`, `warning` | `info` |
| `Scheme` | `http`, `https` | `https` |
| `FontSource` | `upload`, `google` | None, set by the server |

Values are lowercase.
Another value returns `400` with a message that lists the valid values.

## TypeScript

The `cuhlumn-api` crate exports every wire type to TypeScript in `console/src/types/api/`.
Value types are `string` aliases, and optional values are `T | null`.
