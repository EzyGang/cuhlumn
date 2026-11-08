# Share images

The server draws a 1200×630 PNG share image for each site and each entry.
Social networks and chat apps show this image when someone shares a link.

| Address | Title | Subtitle | Bottom right |
| --- | --- | --- | --- |
| `/og.png` | Site title | Site description | Empty |
| `/og/{slug}.png` | Entry title | Entry description | Date and reading time, for published posts, when `card_meta` is `true` |

## Layout

- A solid background with a band on the left edge and a disc and ring in the top right corner
- The logo or the first letter of the site title on a badge, at the top left
- The site title next to the badge, unless it is the same as the image title
- The title in large bold text
- The subtitle in a softer color below the title, on one or two lines
- A thin rule, then the canonical domain at the bottom left and the meta text at the bottom right

The title and the subtitle are centered vertically between the top row and the rule.
On an accent badge, the letter is white or black, whichever has more contrast with the accent.

## Card styles

The `brand.card_style` setting selects the colors of the image.

| `card_style` | Background | Text | Shapes and badge |
| --- | --- | --- | --- |
| `dark` | Near black, the look of the first release | Light grey | Accent |
| `light` | Off-white | Near black | Accent |
| `accent` | The accent color | White or black, whichever has more contrast with the accent | The text color, partly transparent |

## Card meta

With `brand.card_meta` set to `false`, post images have no date and no reading time.
The bottom right corner then stays empty.

## Title size

The server estimates the width of each line from a fixed width factor for each character of the card font.
It tries title sizes from 88 to 48 pixels in steps of 2 pixels.
At each size, it breaks the title into lines at spaces.
It selects the largest size at which all lines fit the width and the height of the title area.

The subtitle has a fixed size of 30 pixels.
The server places the subtitle first and gives the remaining height of the title area to the title.
A word that is wider than one line breaks between characters.
When the title does not fit at 48 pixels, the server keeps the lines that fit and ends the last line with an ellipsis.
A subtitle that is longer than two lines, a long site title, and a long domain also end with an ellipsis.

## Logo

The image contains the site logo only when all these conditions are true:

- The `logo` is `/media/{name}`, or an absolute media address on the canonical domain.
- The file exists in the site media.
- The file is PNG, JPEG, or GIF.

In other cases the image shows the first letter of the site title.

## Fonts

The image uses the font list `Inter, Arial, DejaVu Sans, sans-serif`.
The Docker image contains Inter.

The server loads the system fonts and every font of the font library.
When Inter has no glyph for a character, the server uses a library font or a system font that has it.
Add fonts for scripts such as Japanese, Arabic, or Devanagari to show titles in those scripts.

### Font library

The font library is the `fonts/` folder in the data folder.
Manage it on the **Fonts** page of the console, or through the [fonts API](../reference/fonts.md).
The library also supplies [blog fonts](appearance.md#blog-font).
Share images still use Inter first, and library fonts add glyphs for other scripts.

- Upload a TTF, OTF, or TTC file of up to 20 MiB.
- Or import a family from Google Fonts with a `https://fonts.google.com/specimen/{Family}` address or a `https://fonts.googleapis.com/css2?family=…` address.
- An import downloads at most 20 files, each up to 20 MiB, only from `fonts.gstatic.com`.

After each upload, import, or deletion, the server loads the fonts again and clears every cached page and image.
The next request draws the images with the new fonts.

## Custom share images

- The site `brand.og_image` replaces `/og.png` on the home page, tag pages, and the 404 page.
- The post `og_image` replaces `/og/{slug}.png` for that post.

Custom images do not get `og:image:width` and `og:image:height` tags.

## Versions

Pages link to the share images with a `?v=` value:

- `/og.png?v={site updated_at}`
- `/og/{slug}.png?v={the later of post and site updated_at}`

A change to the site or the post gives a new address, so social networks fetch the new image.
