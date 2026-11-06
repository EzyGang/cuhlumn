# Appearance

The `brand` object of a site contains its look settings.
The console shows these settings in the **Appearance** section.
All fields are optional.

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

Image and link addresses can be absolute (`https://…`) or root-relative (`/media/…`).

## Accent color

The accent accepts `#rgb` or `#rrggbb`.
The server calculates link colors that stay readable:

1. For the light theme, the server mixes the accent toward black until the contrast is at least 4.6:1 against the light backgrounds.
2. For the dark theme, the server mixes the accent toward white until the contrast is at least 4.6:1 against the dark backgrounds.
3. Buttons use the light link color with white text.

Any accent therefore gives links that meet the WCAG AA contrast level.

## Font

| Value | Result |
| --- | --- |
| `serif` | Post text in a book serif: Iowan Old Style, Palatino Linotype, Palatino, Book Antiqua, Charter, or Georgia |
| `sans` | Post text in the system sans-serif font |

Headings and the interface always use the system sans-serif font.
The pages load no web fonts.

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

## Themes

Every page has a light and a dark theme.
The page follows the system preference until the reader selects the theme button in the header.
The button switches between the light and the dark theme.
The browser stores the choice in `localStorage` under the key `cuhlumn:theme`.
A small script in the page head applies the stored choice before the page shows.

## Head HTML

The `head` setting adds raw HTML to every page, for example analytics scripts or verification tags.
The server does not check this HTML.
