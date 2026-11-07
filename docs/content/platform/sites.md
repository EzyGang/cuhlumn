# Sites and domains

One Cuhlumn server hosts many sites.
Each site is one blog with one canonical domain.

## Canonical domain

The canonical domain identifies the site in the API path, such as `/api/sites/blog.example.com`.
The server writes this domain in every absolute address: canonical links, the feed, the sitemap, and share images.
You cannot rename a site.
To move a blog to a new domain, create a new site and copy the posts.

The server normalizes domains:

- Uppercase letters become lowercase.
- Trailing dots are removed.
- The `Host` header port is ignored.

## Aliases

An alias is another domain of the same site, such as `www.blog.example.com`.
A request to an alias returns `301` to the same path and query on the canonical domain.

- A domain can belong to only one site, as a canonical domain or as an alias. A conflict returns `409`.
- The site's own domain is removed from its alias list.
- Repeated aliases are removed.

## Unknown domains

Without a console domain, a domain that is not a site or an alias serves the console.
With a console domain, a request to any other unknown domain returns `404` with the text `No blog lives at this address.`
The `/api` routes still answer on every domain.

## Console domain

The console domain serves the management console instead of a blog.
The admin sets it on the **Server settings** page of the console, or with `CUHLUMN_CONSOLE_HOST`.

- With a console domain, only that domain serves the console.
- Without a console domain, every domain that is not a site domain or an alias serves the console.
- A site write with the console domain as its domain or alias returns `409` with the code `conflict`.
- A console domain that belongs to a site returns `409` from `PUT /api/settings`.
- With an [admin listener](../operations/configuration.md#admin-listener), no domain serves the console, and site domains and aliases can use any name.

## Site settings

| Setting | Effect |
| --- | --- |
| `title` | The blog name in the header, the page titles, the feed, and the share images |
| `description` | The home page lead text, the meta description, the feed subtitle, and `llms.txt` |
| `aliases` | Other domains that redirect to the canonical domain |
| `author`, `author_url` | The author in the post meta line, the JSON-LD, and the feed |
| `project_name`, `project_url` | The product link in the header and the call to action after each post |
| `cta` | The call-to-action text above the product button |
| `lang` | The `lang` attribute, `inLanguage` in JSON-LD, and `xml:lang` in the feed |
| `home` | A page slug to show at `/` |
| `indexnow_key` | The key for IndexNow, served at `/{key}.txt` |
| `indexnow` | Sends IndexNow notifications on publication, `true` by default |
| `announcement` | The [announcement bar](announcements.md) above every page |
| `brand` | The [appearance](appearance.md) settings |

See [Sites reference](../reference/sites.md) for types, defaults, and limits.

## Delete a site

`DELETE /api/sites/{host}` removes the site with its aliases, posts, pages, and media records.
The server also removes the media folder of the site.
The operation cannot be undone.
