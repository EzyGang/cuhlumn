# Management console

The management console is a web interface for every site on the server.
The server embeds the console and serves it only on the domain in `CUHLUMN_CONSOLE_HOST`.

## Log in

1. Open the console domain in a browser.
2. Enter the admin token in the **Admin token** field.
3. Select **Log in**.

The server sets a session cookie for the console domain.
The session expires after 7 days.
**Log out** in the top bar ends the session.
A change of `CUHLUMN_TOKEN` ends all sessions.

## Screens

| Address | Screen | Actions |
| --- | --- | --- |
| `/` | Your blogs | List all sites, open a site, create a site |
| `/new-site` | Site settings | Create a site |
| `/sites/{host}` | Site settings | Change the settings and the appearance, delete the site |
| `/sites/{host}/posts` | Posts and pages | Search by title or slug, filter by status and kind, open the public page |
| `/sites/{host}/new-post` | Editor | Write a new post or page |
| `/sites/{host}/posts/{slug}` | Editor | Edit, publish, unpublish, or delete an entry |
| `/sites/{host}/media` | Media library | Upload, copy the address, delete |

The top bar has a site switcher.
Each site has the tabs **Posts**, **Media**, and **Settings**.

## Site settings

| Section | Fields |
| --- | --- |
| Domains | Host (fixed after creation), aliases, IndexNow key |
| General | Title, description, call to action, author, author address, project name, project address, language, home page |
| Appearance | Accent with a color picker, font, logo, favicon, share image, header links, head HTML, footer, credit |
| Delete site | Deletes the site with all posts, pages, and media after a confirmation |

**Pick from media** next to the logo, favicon, and share image fields opens the media library.
The selected image goes into the field as `/media/{name}`.

## Editor

The editor contains the title, the slug, the kind, the description, the tags, the cover image, the share image, and the Markdown body.

- The console suggests a slug from the title. The slug cannot change after the first save.
- The preview pane renders the body with the server Markdown renderer 300 ms after the last change.
- The preview shows the word count and the table of contents.
- The preview runs in a sandboxed frame, so scripts and styles in the body cannot affect the console.

| Entry status | Buttons |
| --- | --- |
| Draft or new | **Save draft**, **Publish** |
| Published | **Save**, **Unpublish** |

**Delete post** asks for a confirmation.
The home page cannot be deleted.

If the session ends during an edit, the console keeps the unsaved form.
After the next login, the console restores it when you open the same entry.

## Media library

- Drop files on the upload area or select **Choose files**.
- Accepted types are PNG, JPEG, WebP, GIF, and AVIF, up to 16 MiB each.
- Each tile shows the type, the size, and the upload time.
- **Copy** puts the absolute address on the clipboard.

## Errors

When the server rejects a value, the console shows the message below the field.
Other errors appear in a notification.
If a screen cannot load its data, it shows the error and a **Retry** button.

## Security

The console responses have these headers:

- `Content-Security-Policy: default-src 'self'; img-src 'self' https: data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'` (with `http:` added to `img-src` when `CUHLUMN_SCHEME=http`)
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: same-origin`

The console stores no token in the browser.
