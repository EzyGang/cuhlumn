# Share images

The server draws a 1200×630 PNG share image for each site and each entry.
Social networks and chat apps show this image when someone shares a link.

| Address | Title | Subtitle | Bottom right |
| --- | --- | --- | --- |
| `/og.png` | Site title | Site description | Empty |
| `/og/{slug}.png` | Entry title | Entry description | Date and reading time, for published posts |

## Layout

- A solid dark background with an accent band on the left edge and an accent disc and ring in the top right corner
- The logo or the first letter of the site title on an accent badge, at the top left
- The site title next to the badge, unless it is the same as the image title
- The title in large bold text
- The subtitle in light gray below the title, on one or two lines
- A thin rule, then the canonical domain at the bottom left and the meta text at the bottom right

The title and the subtitle are centered vertically between the top row and the rule.
The letter on the badge is white or black, whichever has more contrast with the accent.

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

The image uses Inter, then Arial or DejaVu Sans.
The Docker image contains Inter.
Set `CUHLUMN_FONTS` to a folder to load more fonts.

## Custom share images

- The site `brand.og_image` replaces `/og.png` on the home page, tag pages, and the 404 page.
- The post `og_image` replaces `/og/{slug}.png` for that post.

Custom images do not get `og:image:width` and `og:image:height` tags.

## Versions

Pages link to the share images with a `?v=` value:

- `/og.png?v={site updated_at}`
- `/og/{slug}.png?v={the later of post and site updated_at}`

A change to the site or the post gives a new address, so social networks fetch the new image.
