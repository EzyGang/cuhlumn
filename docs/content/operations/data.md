# Data and backups

The server keeps all data in the folder from `CUHLUMN_DATA`.

```text
data/
  cuhlumn.db          SQLite database
  cuhlumn.db-wal      SQLite write-ahead log
  cuhlumn.db-shm      SQLite shared memory
  media/
    blog.example.com/
      ba7816bf8f01cfea414140de5dae2223.png
```

## Database

| Table | Contents |
| --- | --- |
| `sites` | One row for each site, with the settings and the appearance |
| `site_aliases` | One row for each alias domain |
| `posts` | Posts and pages with their Markdown bodies |
| `media` | One record for each uploaded image |
| `sessions` | Hashed console sessions |

- The database uses WAL mode, foreign keys, and a busy timeout of 5 seconds.
- Deleting a site deletes its aliases, posts, and media records.
- Rendered pages are not stored. The server renders them again after a restart.

## Migrations

The database stores its schema version in `PRAGMA user_version`.
At startup the server applies each newer migration in its own transaction.
Do not open the database with an older server version after a migration.

## Backups

Back up the database and the media folder together.

1. Copy the database with the SQLite online backup, for example `sqlite3 data/cuhlumn.db ".backup backup.db"`.
2. Copy the `media` folder.

A plain file copy of `cuhlumn.db` while the server runs can miss data in the WAL file.
Stop the server first if you copy the files directly.

## Restore

1. Stop the server.
2. Put the database file and the `media` folder in the data folder.
3. Start the server.
