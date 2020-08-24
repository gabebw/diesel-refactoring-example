CREATE TABLE posts (
  id INTEGER PRIMARY KEY NOT NULL,
  title TEXT,
  body TEXT,
  created_at DATETIME DEFAULT current_timestamp NOT NULL
);
