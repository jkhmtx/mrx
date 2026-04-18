use std::env;
use std::path::Path;
use std::path::PathBuf;

use exn::ResultExt;
use mrx_utils::Attrname;
use mrx_utils::fs::AbsolutePathBuf;
use mrx_utils::fs::mk_dir;
use mrx_utils::graph::NodeId;
use mrx_utils::nix_store_path::NixStorePath;
use rusqlite::OptionalExtension;
use rusqlite::named_params;
use rusqlite::{
    Connection,
    Result,
    Statement,
};
use rusqlite_migration::M;
use rusqlite_migration::Migrations;
use thiserror::Error as ThisError;

use crate::unix_seconds::UnixSeconds;

pub type DbResult<T, E> = Result<T, exn::Exn<E>>;

#[derive(Debug, Clone, Copy, ThisError)]
pub enum ConnectError {
    #[error("ConnectError::Environment")]
    Environment,
    #[error("ConnectError::Connect")]
    Connect,
    #[error("ConnectError::Layout")]
    Layout,
    #[error("ConnectError::Migrations")]
    Migrations,
    #[error("ConnectError::Pragmas")]
    Pragmas,
}

#[derive(Debug, ThisError)]
pub enum DbError {
    #[error("DbError::Connect")]
    Connect,
    #[error("DbError::Query\n\n{0}")]
    Query(DbQueryError),
    #[error("DbError::Statement")]
    Statement,
}

impl DbError {
    fn query_error_with(statement: Statement<'_>) -> impl FnOnce(rusqlite::Error) -> Self {
        move |e| {
            Self::Query(DbQueryError(
                e,
                statement
                    .expanded_sql()
                    .unwrap_or_else(|| "Failed to expand".to_string()),
            ))
        }
    }
}

#[derive(Debug)]
pub struct DbQueryError(rusqlite::Error, String);

impl std::fmt::Display for DbQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = &self.0;
        let query = &self.1;
        f.write_str(&format!(
            r"
DbQueryError:
{err}

SQL:
{query}
"
        ))
    }
}

static DEFAULT_DATABASE_PATH: &str = "~/.cache/mrx/main.db";

impl core::error::Error for DbQueryError {}

pub(crate) fn get_connection() -> DbResult<Connection, ConnectError> {
    let database_path = match env::var("MRX_DATABASE_PATH").map(PathBuf::from) {
        Ok(path) => Ok(path),
        Err(env::VarError::NotPresent) => Ok(PathBuf::from(DEFAULT_DATABASE_PATH)),
        Err(e) => Err(e),
    }
    .or_raise(|| ConnectError::Environment)?;

    ensure_db_path(&database_path)?;

    let connection = {
        let mut connection = Connection::open(&database_path).or_raise(|| ConnectError::Connect)?;

        run_migrations(&mut connection)?;
        update_pragmas(&mut connection)?;

        connection
    };

    Ok(connection)
}

fn ensure_db_path(path: &Path) -> DbResult<(), ConnectError> {
    if path.extension().is_some()
        && let Some(parent) = path.parent()
    {
        mk_dir(parent)
    } else {
        mk_dir(path)
    }
    .or_raise(|| ConnectError::Layout)
}

const MIGRATIONS_SLICE: &[M] = &[
    M::up(include_str!("../../../sql/migrations/00-init.sql")),
    M::up(include_str!(
        "../../../sql/migrations/01-create-node-and-alias-tables.sql"
    )),
    M::up(include_str!(
        "../../../sql/migrations/02-create-store-table.sql"
    )),
];

const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

fn run_migrations(connection: &mut Connection) -> DbResult<(), ConnectError> {
    MIGRATIONS
        .to_latest(connection)
        .or_raise(|| ConnectError::Migrations)
}

fn update_pragmas(connection: &mut Connection) -> DbResult<(), ConnectError> {
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .and_then(|()| connection.pragma_update(None, "foreign_keys", "ON"))
        .or_raise(|| ConnectError::Pragmas)
}

/// # Errors
/// See [`DbError`] and [`ConnectError`].
pub fn get_mtime(
    connection: &Connection,
    node_id: &NodeId,
) -> DbResult<Option<UnixSeconds>, DbError> {
    let (sql, params) = match &node_id {
        NodeId::Attrname(name) => (
            "
                SELECT
                    mtime
                FROM
                    node
                    JOIN alias ON alias.node_id = node.id
                WHERE
                    alias.alias = :name;
                ",
            named_params! {
                ":name": name.to_string()
            },
        ),

        NodeId::Path(path) => (
            "
                SELECT
                    mtime
                FROM
                    node
                WHERE
                    path = :path;
                ",
            named_params! {
                ":path": path.to_string()
            },
        ),
    };

    let mut statement = connection.prepare(sql).or_raise(|| DbError::Statement)?;

    statement
        .query_row(params, |row| {
            let w: i64 = row.get(0)?;
            Ok(UnixSeconds::from(w))
        })
        .optional()
        .map_err(DbError::query_error_with(statement))
        .map_err(exn::Exn::from)
}

/// # Errors
/// See [`DbError`] and [`ConnectError`].
pub fn set_node_mtime(
    connection: &Connection,
    path: &AbsolutePathBuf,
    mtime: UnixSeconds,
) -> DbResult<i64, DbError> {
    let mut statement = connection
        .prepare(
            "
INSERT INTO node (path, mtime)
    VALUES (:path, :mtime)
ON CONFLICT (path)
    DO UPDATE SET
        mtime = :mtime;
            ",
        )
        .or_raise(|| DbError::Statement)?;

    statement
        .insert(named_params! {
            ":path": path.to_string(),
            ":mtime": mtime.to_sql(),
        })
        .map_err(DbError::query_error_with(statement))
        .map_err(exn::Exn::from)
}

/// # Errors
/// See [`DbError`] and [`ConnectError`].
pub fn set_alias_mtime(
    connection: &Connection,
    alias: &Attrname,
    path: &AbsolutePathBuf,
    mtime: UnixSeconds,
) -> DbResult<(), DbError> {
    let id = set_node_mtime(connection, path, mtime)?;

    let mut statement = connection
        .prepare(
            "
INSERT INTO alias (alias, node_id)
    VALUES (:alias, :id)
ON CONFLICT (alias)
    DO NOTHING;
",
        )
        .or_raise(|| DbError::Statement)?;

    statement
        .execute(named_params! {
            ":alias": alias.to_string(),
            ":id": id,
        })
        .map(|_| ())
        .map_err(DbError::query_error_with(statement))
        .map_err(exn::Exn::from)
}

/// # Errors
/// See [`DbError`] and [`ConnectError`].
pub fn get_store_bin_path(
    connection: &Connection,
    alias: &Attrname,
) -> DbResult<Option<NixStorePath>, DbError> {
    let mut statement = connection
        .prepare(
            "
        SELECT
            store_path
        FROM
            store
            JOIN alias ON alias.id = store.alias_id
        WHERE
            alias.alias = :alias;
        ",
        )
        .or_raise(|| DbError::Statement)?;

    statement
        .query_row(
            named_params! {
                ":alias": alias.to_string()
            },
            |row| row.get(0).map(NixStorePath::new),
        )
        .optional()
        .map_err(DbError::query_error_with(statement))
        .map_err(exn::Exn::from)
}

#[derive(Debug, Clone, Copy, ThisError)]
pub enum WriteStoreError {
    #[error("WriteStoreError:DbError")]
    DbError,
    #[error("WriteStoreError::MissingAlias")]
    MissingAlias,
}

impl WriteStoreError {
    #[must_use]
    pub fn is_missing_alias(&self) -> bool {
        matches!(self, WriteStoreError::MissingAlias)
    }
}

type WriteStoreResult = DbResult<(), WriteStoreError>;

/// # Errors
/// Errors if there is an underlying database error (see [`DbResult`]), or if the alias-to-write doesn't exist in the database.
/// In the missing alias case, a retry after writing the alias is suitable for error handling.
pub fn write_store(
    connection: &Connection,
    alias: &Attrname,
    store_path: &NixStorePath,
) -> WriteStoreResult {
    let mut statement = connection
        .prepare(
            "
INSERT INTO store (alias_id, store_path)
    VALUES ((
            SELECT
                id
            FROM
                alias
            WHERE
                alias = :alias),
            :store_path)
ON CONFLICT(alias_id) 
DO UPDATE SET store_path = excluded.store_path;
",
        )
        .or_raise(|| DbError::Statement)
        .map_err(|e| e.raise(WriteStoreError::DbError))?;

    statement
        .insert(named_params! {
            ":alias": alias.to_string(),
            ":store_path": store_path.clone().into_string(),
        })
        .map(|_| ())
        .map_err(DbError::query_error_with(statement))
        .map_err(exn::Exn::from)
        .map_err(|e| e.raise(WriteStoreError::DbError))
}
