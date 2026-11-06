# Quick start

This page creates a blog, publishes a post, and opens the console.
The examples use `blog.example.com` as the blog domain and `admin.example.com` as the console domain.

## 1. Create a blog

```sh
curl -X PUT https://blog.example.com/api/sites/blog.example.com \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Harbor Notes",
    "description": "How a two-person team runs Harbor, a small hosting service.",
    "aliases": ["www.blog.example.com"],
    "author": "Mara Lind",
    "project_name": "Harbor",
    "project_url": "https://harbor.example.com",
    "cta": "Harbor runs your app on a server that you own.",
    "brand": { "accent": "#0e7490" }
  }'
```

The answer is the complete [`Site`](../reference/sites.md) object, with a generated IndexNow key.

## 2. Save a draft

The slug comes from the address.

```sh
curl -X PUT https://blog.example.com/api/sites/blog.example.com/posts/sqlite-backups \
  -H "Authorization: Bearer $CUHLUMN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "How we back up SQLite",
    "description": "Our nightly routine for one database file.",
    "body": "We back up **one SQLite file** every night.\n\n## Why one file\n\nIt is easy to copy.",
    "tags": ["Databases"]
  }'
```

The status is `draft`, so `https://blog.example.com/sqlite-backups/` returns `404`.

## 3. Publish the post

Send the same request with `"status": "published"`.
The post appears on the home page, in the feed, and in the sitemap.
The server also sends the post address to IndexNow.

## 4. Open the console

1. Open `https://admin.example.com/` in a browser.
2. Enter the admin token in the token field.
3. Select **Log in**.

The console shows the blog, its posts, its media, and its settings.
See [Management console](../console/index.md).

## 5. Look at the blog

| Address | Contents |
| --- | --- |
| `https://blog.example.com/` | Home page with the post list |
| `https://blog.example.com/sqlite-backups/` | The post |
| `https://blog.example.com/sqlite-backups.md` | The post as Markdown |
| `https://blog.example.com/feed.xml` | Atom feed |
| `https://blog.example.com/og/sqlite-backups.png` | Share image of the post |
| `https://www.blog.example.com/` | Redirect to the canonical domain |
