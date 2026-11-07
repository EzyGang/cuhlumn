# Appearance

The `brand` object of a site contains its look settings.
The console shows these settings in the **Appearance** section.
All fields are optional.
The defaults give the look of the first release.

## Settings

| Field | Default | Effect |
| --- | --- | --- |
| `accent` | `#2563eb` | The accent color for links, buttons, the progress bar, the call-to-action bar, the favicon, and the share images |
| `font` | `serif` | The text font of posts: `serif` or `sans` |
| `logo` | None | The image in the header and on the share images |
| `favicon` | None | The browser icon |
| `og_image` | None | The share image for the home page and other site pages |
| `links` | None | Up to 8 header links with a label and an address |
| `head` | Empty | Raw HTML at the end of every page `<head>`, up to 20,000 bytes |
| `footer` | Empty | A text line in the footer, up to 500 characters |
| `credit` | `true` | Shows "Published with Cuhlumn" in the footer |
| `palette` | `paper` | The colors of backgrounds and text |
| `theme` | `system` | The theme that the page shows first |
| `theme_toggle` | `true` | Shows the theme button in the header |
| `width` | `normal` | The width of the text column |
| `home_layout` | `list` | The layout of the post lists |
| `social` | None | Up to 12 profile links, shown as icons |
| `social_in_header` | `false` | Shows the profile icons in the header too |
| `code_theme` | `github` | The colors of highlighted code |
| `card_style` | `dark` | The colors of the share images |
| `card_meta` | `true` | Shows the date and the reading time on share images |

Image and link addresses can be absolute (`https://…`) or root-relative (`/media/…`).
See [Share images](share-images.md) for `card_style` and `card_meta`.

## Palettes

Each palette has a light and a dark variant.

| Value | Look | Minimum text contrast |
| --- | --- | --- |
| `paper` | Off-white paper and a dark grey night, the look of the first release | 4.5:1 |
| `slate` | Cool blue greys | 4.5:1 |
| `sepia` | Warm cream and brown | 4.5:1 |
| `mono` | Neutral greys without a tint | 4.5:1 |
| `contrast` | Black and white with strong dividers | 7:1 |

## Accent color

The accent accepts `#rgb` or `#rrggbb`.
The server calculates link colors that stay readable on the backgrounds of the palette:

1. For the light variant, the server mixes the accent toward black until the contrast reaches the target.
2. For the dark variant, the server mixes the accent toward white until the contrast reaches the target.
3. Buttons use the light link color with white text.

The target is 4.6:1, or 7.1:1 in the `contrast` palette.
Any accent therefore gives links that meet the WCAG AA contrast level.

## Font

| Value | Result |
| --- | --- |
| `serif` | Post text in a book serif: Iowan Old Style, Palatino Linotype, Palatino, Book Antiqua, Charter, or Georgia |
| `sans` | Post text in the system sans-serif font |

Headings and the interface always use the system sans-serif font.
The pages load no web fonts.

## Themes

| `theme` | First theme |
| --- | --- |
| `system` | The light or dark theme from the `prefers-color-scheme` setting of the reader |
| `light` | Always the light theme |
| `dark` | Always the dark theme |

A forced theme is the `data-theme` attribute of the root element, so the first paint already uses it.
The `color-scheme` and `theme-color` tags follow the theme setting.

With `theme_toggle` set to `true`, the header has a theme button:

- The button switches between the light and the dark theme.
- The browser stores the choice in `localStorage` under the key `cuhlumn:theme`.
- A small script in the page head applies the stored choice before the page shows.
- The stored choice wins over the `theme` setting.

With `theme_toggle` set to `false`, the page has no button and no theme script.
The page then always shows the `theme` setting.

## Width

| `width` | Text column |
| --- | --- |
| `narrow` | 38rem |
| `normal` | 42rem |
| `wide` | 52rem |

## Home layouts

The `home_layout` applies to the post list of the home page and to tag pages.
The "More posts" list of a post and the 404 page always use `list`.

| `home_layout` | Result |
| --- | --- |
| `list` | Date, title, and summary of each post |
| `compact` | One line for each post with the date and the title |
| `cards` | A grid of cards with the cover image, the date, the title, and the summary |

In the `cards` layout, a post without a cover gets a card without an image.

## Social links

Each item of `social` has a `network` and a `url`.
The page shows one icon for each link, in the order of the list.

| Rule | Value |
| --- | --- |
| Count | At most 12 |
| Repetition | The same network with the same address is not allowed |
| `url` length | At most 500 characters |
| `url` for `email` | `mailto:` with one address, without `?`, `,`, or `;` |
| `url` for other networks | An absolute `https://` address |

The networks are `github`, `gitlab`, `codeberg`, `mastodon`, `bluesky`, `x`, `threads`, `linkedin`, `youtube`, `instagram`, `discord`, `telegram`, and `email`.

- The icons are in the footer. With `social_in_header`, they are also in the header.
- Each link has the network name as its accessible name, such as "GitHub".
- Each link has `rel="me noopener"`, so services such as Mastodon can verify the profile.
- Every link except `email` goes into the JSON-LD `sameAs` list of the publisher. See [Search and AI discovery](discovery.md#structured-data).

## Code highlighting

The server highlights fenced code blocks that name a known language, such as ` ```rust `.
Blocks without a language, or with an unknown language, stay plain.

Each token is a `<span>` with classes from its syntax scope, with the prefix `hl-`.
For example, a Rust `fn` is `<span class="hl-storage hl-type hl-function hl-rust">`.
The site stylesheet gives the classes their colors.

Each code theme uses one color set for the light theme and one for the dark theme:

| `code_theme` | Light theme | Dark theme |
| --- | --- | --- |
| `github` | InspiredGitHub | base16 Eighties dark |
| `solarized` | Solarized light | Solarized dark |
| `ocean` | base16 Ocean light | base16 Ocean dark |
| `none` | No colors | No colors |

The server keeps the hue of each token color and changes its lightness until it has a contrast of at least 4.6:1 against the code background.
With `none`, the spans stay in the HTML, but the stylesheet gives them no colors.

## Logo and favicon

- With a `logo`, the header shows the logo at 28×28 pixels. Without a logo, the header shows the first letter of the title on an accent badge.
- With a `favicon`, the page links to it. Without a favicon, the page uses the logo. Without both, the page uses the generated `/favicon.svg`.
- The generated favicon is a round disc in the accent color with the first letter of the title in a serif font. The letter is white or black, whichever has more contrast with the accent.
- The share images draw the logo only when it is a PNG, JPEG, or GIF file from the site's own media. See [Share images](share-images.md).

## Header

The header navigation shows these items in order:

1. Published pages, except the home page
2. The `links` from the appearance settings
3. The product link, when `project_url` is set

After the navigation, the header shows the profile icons when `social_in_header` is `true`, and then the theme button.

## Head HTML

The `head` setting adds raw HTML to every page, for example analytics scripts or verification tags.
The server does not check this HTML.

## Example

```json
{
  "accent": "#0e7490",
  "palette": "slate",
  "theme": "system",
  "width": "wide",
  "home_layout": "cards",
  "code_theme": "solarized",
  "social": [
    { "network": "github", "url": "https://github.com/maralind" },
    { "network": "email", "url": "mailto:mara@harbor.example.com" }
  ],
  "social_in_header": true
}
```
