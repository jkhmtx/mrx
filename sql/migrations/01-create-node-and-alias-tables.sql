CREATE TABLE IF NOT EXISTS node (
    id integer PRIMARY KEY autoincrement,
    path text CHECK (path LIKE '/%') UNIQUE NOT NULL,
    mtime datetime NOT NULL
);

CREATE TABLE IF NOT EXISTS alias (
    id integer PRIMARY KEY,
    alias text CHECK (alias LIKE '_.%') UNIQUE NOT NULL,
    node_id integer UNIQUE NOT NULL,
    FOREIGN KEY (node_id) REFERENCES node (id))
STRICT;
