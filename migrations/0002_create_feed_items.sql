CREATE TABLE feed_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feed_id INTEGER NOT NULL,
  item_url TEXT NOT NULL,
  title TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,

  UNIQUE(feed_id, item_url),
  FOREIGN KEY(feed_id) REFERENCES feeds(id) ON DELETE CASCADE
);
