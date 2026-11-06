# Media

Each site has its own media library for images.

## Accepted images

| Type | Content-Type | File extension |
| --- | --- | --- |
| PNG | `image/png` | `.png` |
| JPEG | `image/jpeg` | `.jpg` |
| WebP | `image/webp` | `.webp` |
| GIF | `image/gif` | `.gif` |
| AVIF | `image/avif` | `.avif` |

The first bytes of the file must match the declared type.
SVG and other types are refused with `415`.
The maximum size is 16 MiB.

## Upload

Send the raw bytes with the image `Content-Type`:

```sh
curl -X POST https://blog.example.com/api/sites/blog.example.com/media \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: image/png" \
  --data-binary @logo.png
```

```json
{
  "name": "ba7816bf8f01cfea414140de5dae2223.png",
  "url": "https://blog.example.com/media/ba7816bf8f01cfea414140de5dae2223.png"
}
```

## Names

The file name is the first 32 hexadecimal characters of the SHA-256 hash of the content, with the extension.
The same image therefore always gets the same name.
A repeated upload of the same image returns the same name and does not write the file again.

## Use an image

Use the root-relative path `/media/{name}` or the absolute `url` in these places:

- Markdown, for example `![Diagram](/media/{name})`
- The post `cover` and `og_image`
- The site `logo`, `favicon`, and `og_image`

A root-relative path works behind any proxy and on any port.

## Public address

`/media/{name}` on the site domain serves the file with these headers:

- `Cache-Control: public, max-age=31536000, immutable`
- `X-Content-Type-Options: nosniff`

An alias domain redirects to the canonical domain.
A missing file returns `404`.

## Delete

`DELETE /api/sites/{host}/media/{name}` removes the record and the file.
The server does not check whether a post still uses the image.
