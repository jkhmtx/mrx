use std::env;
use std::env::VarError;

use mrx_utils::Attrname;
use mrx_utils::fs::AbsolutePathBuf;
use mrx_utils::graph::NodeId;
use mrx_utils::nix_store_path::NixStorePath;
use rusqlite::OptionalExtension;
use rusqlite::named_params;
use rusqlite::{
    Connection,
    Result,
    Statement,
};
use thiserror::Error;

use crate::unix_seconds::UnixSeconds;

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Environment error: {0}")]
    Environment(#[from] VarError),
    #[error("{0}")]
    Connect(#[from] rusqlite::Error),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to connect: {0}")]
    Connect(#[from] ConnectError),
    #[error("Failed to query: {0}")]
    Query(DbQueryError),
    #[error("Failed to build statement: {0}")]
    Statement(#[from] rusqlite::Error),
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

impl core::error::Error for DbQueryError {}

fn get_connection() -> Result<Connection, ConnectError> {
    let database_path = env::var("DATABASE_PATH")?;

    Ok(Connection::open(&database_path)?)
}

/// # Errors
/// TODO
pub fn get_mtime(node_id: &NodeId) -> Result<Option<UnixSeconds>, DbError> {
    let connection = get_connection()?;

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

    let mut statement = connection.prepare(sql)?;
    statement
        .query_row(params, |row| {
            let w: i64 = row.get(0)?;
            Ok(UnixSeconds::from(w))
        })
        .optional()
        .map_err(DbError::query_error_with(statement))
}

/// # Errors
/// TODO
pub fn set_node_mtime(path: &AbsolutePathBuf, mtime: UnixSeconds) -> Result<i64, DbError> {
    let connection = get_connection()?;

    let mut statement = connection.prepare(
        "
INSERT INTO node (path, mtime)
    VALUES (:path, :mtime)
ON CONFLICT (path)
    DO UPDATE SET
        mtime = :mtime;
            ",
    )?;

    statement
        .insert(named_params! {
            ":path": path.to_string(),
            ":mtime": mtime.to_sql(),
        })
        .map_err(DbError::query_error_with(statement))
}

/// # Errors
/// TODO
pub fn set_alias_mtime(
    alias: &Attrname,
    path: &AbsolutePathBuf,
    mtime: UnixSeconds,
) -> Result<(), DbError> {
    let connection = get_connection()?;

    let id = set_node_mtime(path, mtime)?;

    let mut statement = connection.prepare(
        "
INSERT INTO alias (alias, node_id)
    VALUES (:alias, :id)
ON CONFLICT (alias)
    DO NOTHING;
",
    )?;

    statement
        .execute(named_params! {
            ":alias": alias.to_string(),
            ":id": id,
        })
        .map(|_| ())
        .map_err(DbError::query_error_with(statement))
}

/// # Errors
/// TODO
pub fn get_store_bin_path(alias: &Attrname) -> Result<Option<NixStorePath>, DbError> {
    let connection = get_connection()?;

    let mut statement = connection.prepare(
        "
        SELECT
            store_path
        FROM
            store
            JOIN alias ON alias.id = store.alias_id
        WHERE
            alias.alias = :alias;
        ",
    )?;

    statement
        .query_row(
            named_params! {
                ":alias": alias.to_string()
            },
            |row| row.get(0).map(NixStorePath::new),
        )
        .optional()
        .map_err(DbError::query_error_with(statement))
}

#[derive(Debug, Error)]
pub enum WriteStoreError {
    #[error(transparent)]
    DbError(DbError),
    #[error("MissingAlias")]
    MissingAlias,
}

impl From<ConnectError> for WriteStoreError {
    fn from(value: ConnectError) -> Self {
        WriteStoreError::DbError(value.into())
    }
}

impl From<DbError> for WriteStoreError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::Statement(_) | DbError::Connect(_) => value.into(),
            DbError::Query(e) => {
                if let Some(rusqlite::ErrorCode::ConstraintViolation) = e.0.sqlite_error_code() {
                    Self::MissingAlias
                } else {
                    WriteStoreError::DbError(DbError::Query(e))
                }
            }
        }
    }
}

type WriteStoreResult = Result<(), WriteStoreError>;

/// # Errors
/// TODO
pub fn write_store(alias: &Attrname, store_path: &NixStorePath) -> WriteStoreResult {
    let connection = get_connection()?;

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
        .map_err(DbError::from)?;

    Ok(statement
        .insert(named_params! {
            ":alias": alias.to_string(),
            ":store_path": store_path.clone().into_string(),
        })
        .map(|_| ())
        .map_err(DbError::query_error_with(statement))?)
}
