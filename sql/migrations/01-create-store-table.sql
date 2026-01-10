CREATE TABLE IF NOT EXISTS store (
    id integer PRIMARY KEY,
    alias_id integer UNIQUE NOT NULL,
    store_path text CHECK (store_path LIKE '/nix/store/%') UNIQUE NOT NULL,
    kind text CHECK (kind IN ('build', 'run')) UNIQUE NOT NULL,
    FOREIGN KEY (alias_id) REFERENCES alias (id))
STRICT;

