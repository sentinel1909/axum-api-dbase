-- Add migration script here

CREATE TABLE test(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  date TEXT NOT NULL,
  message TEXT NOT NULL
);
