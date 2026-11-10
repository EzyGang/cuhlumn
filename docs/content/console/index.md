# Management console

The management console is a web interface for every site on the server.
The server embeds the console in the binary.

- With a console domain, only that domain serves the console.
- Without a console domain, every domain that is not a site domain or an alias serves the console.
- With `CUHLUMN_ADMIN_ADDR`, only the admin port serves the console, on every host name, and the public port never does.

Set the console domain on the **Server settings** page, or with `CUHLUMN_CONSOLE_HOST`.
See [Admin listener](../operations/configuration.md#admin-listener) for the admin port.

The admin port works with HTTPS and without HTTPS.
HTTPS through a TLS proxy that sets `X-Forwarded-Proto: https` is recommended when the network does not encrypt traffic.
Plain HTTP is acceptable on a private network that encrypts traffic itself, such as NetBird or another WireGuard network.
On plain HTTP, browsers block the clipboard API.
The token **Copy** buttons then select the token, and the media library shows the address in a message.

## First launch

When the server has no admin token, `GET /api/health` returns `"needs_setup": true`.
The console then opens the **Setup** page at `/setup`.

1. Enter a token with at least 32 characters in the **Admin token** field, or select **Generate**.
2. Select **Copy**, and keep the token in a safe place. The server stores only a hash and cannot show the token again.
    On plain HTTP pages outside `localhost`, browsers block the clipboard API. The console then shows and selects the token, so you can copy it by hand.
3. Select **Save the token and log in**.

The console calls `POST /api/setup` and starts a session.
After the first setup, the address `/setup` opens the blog list.

## Log in

1. Open the console domain in a browser.
2. Enter the admin token in the **Admin token** field.
3. Select **Log in**.

The server sets a session cookie for the console domain.
The session expires after 7 days.
**Log out** in the top bar ends the session.
A change of the admin token ends all sessions.

## Screens

| Address | Screen | Actions |
| --- | --- | --- |
| `/setup` | Setup | Set the first admin token |
| `/` | Your blogs | List all sites, open a site, create a site |
| `/settings` | Server settings | Change the console domain, the scheme, and the admin token |
| `/fonts` | Font library | Upload fonts, import from Google Fonts, delete fonts |
| `/new-site` | Site settings | Create a site |
| `/sites/{host}` | Site settings | Change the settings, the announcement, and the appearance, delete the site |
| `/sites/{host}/posts` | Posts and pages | Search by title or slug, filter by status and kind, open the public page |
| `/sites/{host}/new-post` | Editor | Write a new post or page |
| `/sites/{host}/posts/{slug}` | Editor | Edit, publish, unpublish, or delete an entry |
| `/sites/{host}/media` | Media library | Upload, copy the address, delete |
| `/sites/{host}/analytics` | Analytics | Read the views, visitors, top pages, referrers, devices, and browsers of a period |

The top bar has a site switcher and the server links **Fonts** and **Server**.
Each site has the tabs **Posts**, **Media**, **Analytics**, and **Settings**.

## Server settings

The **Server settings** page has two sections.

| Section | Fields |
| --- | --- |
| Addresses | Console host, scheme |
| Admin token | New admin token, **Change the token** |

- An empty console host lets every domain that is not a site serve the console.
- A field that an environment variable sets is disabled and shows "Set by CUHLUMN_…", such as "Set by CUHLUMN_SCHEME".
- With an admin port, the console host field is disabled and explains that the console has no console domain.
- After a move of the console host, the page shows the new address. The session stays on the old domain, so log in again at the new address.
- **Change the token** asks for a confirmation. The new token ends all sessions. This browser gets a new session. The CLI and agents need the new token.

See [Server settings reference](../reference/settings.md).

## Font library

The **Fonts** page manages the fonts of the share images and the blog fonts.

- **Choose font files** uploads TTF, OTF, or TTC files, up to 20 MiB each.
- **Import** downloads the fonts of a Google Fonts address, such as `https://fonts.google.com/specimen/Noto+Sans+JP`.
- The table shows the families, the source, the size, and the time of each font.
- **Delete** asks for a confirmation. Share images that need the characters of the font then use other fonts. Blogs that use the family fall back to their system stack.

See [Share images](../platform/share-images.md#fonts).

## Site settings

| Section | Fields |
| --- | --- |
| Domains | Host (fixed after creation), aliases, IndexNow key, **IndexNow** switch |
| General | Title, description, call to action, **Show the call to action on posts by default**, author, author address, project name, project address, language, home page |
| Announcement | **Show the bar**, text, link, link text, tone, start, end, **Dismissible**, and the status |
| Appearance | The groups below |
| Delete site | Deletes the site with all posts, pages, and media after a confirmation |

The **IndexNow** switch turns the notifications of the site on or off.

### Announcement

The status shows **Off**, **Scheduled**, **Live**, or **Ended**.
The bar shows only between the start and end times.
An empty end shows the bar with no end.
See [Announcements](../platform/announcements.md).

### Appearance

| Group | Fields |
| --- | --- |
| Colours | Accent with a color picker, palette, theme, **Theme button** |
| Layout | Width, post list, font, logo, favicon, share image, header links (up to 8) |
| Code | Code colours |
| Social | Profile links, **Show in header** |
| Share image | Generated card, **Date and reading time** |
| Page extras | Head HTML, footer, credit |

The **Font** control in the **Layout** group lists **Serif (system)**, **Sans (system)**, and the families of the font library.
With a family, the **Fallback** choice selects the serif or sans system stack.
A family that is not in the library shows the note "Missing from the font library".
A link opens the **Fonts** page to add fonts.
See [Blog font](../platform/appearance.md#blog-font).

**Choose image** next to the logo, favicon, and share image fields opens a dialog.

- **Upload an image** uploads a PNG, JPEG, WebP, GIF, or AVIF file to the media library and puts it into the field.
- On a saved site, the dialog also lists the media library. Select an image to put it into the field.
- On a new site, the file uploads when you create the site. The field shows the file name until then.

The field gets the image as `/media/{name}`.
If an upload fails during the create, the site exists, and the field of that image stays empty.
See [Appearance](../platform/appearance.md).

## Editor

The editor contains the title, the slug, the kind, the description, the tags, the cover image, the share image, the call to action choice, and the Markdown body.

- The console suggests a slug from the title. The slug cannot change after the first save.
- **Call to action** has three choices: "Site default (shown)" or "Site default (hidden)", **Show**, and **Hide**. The first choice follows the site setting.
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

- `Content-Security-Policy: default-src 'self'; img-src 'self' https: data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'` (with `http:` added to `img-src` when the scheme is `http`)
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: same-origin`

The console stores no token in the browser.
