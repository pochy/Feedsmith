CREATE TABLE feeds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  site_url TEXT NOT NULL,
  feed_description TEXT,
  selector_type TEXT NOT NULL DEFAULT 'css'
    CHECK (selector_type IN ('css', 'xpath')),

  item_selector TEXT NOT NULL,
  title_selector TEXT NOT NULL,
  link_selector TEXT,
  date_selector TEXT,
  content_selector TEXT,

  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
