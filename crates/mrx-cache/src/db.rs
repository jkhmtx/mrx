use std::env;
use std::env::VarError;
use std::str::FromStr as _;

use mrx_utils::Attrname;
use mrx_utils::fs::AbsolutePathBuf;
use mrx_utils::graph::NodeId;
use mrx_utils::nix_store_path::NixStorePath;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{
    FromRow,
    SqlitePool,
};
use thiserror::Error;
use tokio::runtime::Runtime;

use crate::time::Time;

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Environment error: {0}")]
    Environment(#[from] VarError),
    #[error("{0}")]
    Connect(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to connect: {0}")]
    Connect(#[from] ConnectError),
    #[error("Failed to query: {0}")]
    Query(DbQueryError),
}

impl DbError {
    fn query_error_with<T>(query: impl DbQuery<T>) -> impl FnOnce(sqlx::Error) -> Self {
        move |e| Self::Query(DbQueryError(e, query.to_sql()))
    }
}

#[derive(Debug)]
pub struct DbQueryError(sqlx::Error, String);

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

fn get_connection() -> Result<SqlitePool, ConnectError> {
    let database_url = env::var("DATABASE_URL")?;
    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let rt = Runtime::new().unwrap();

    Ok(rt.block_on(SqlitePool::connect_with(options))?)
}

trait DbQuery<TExecuteResult> {
    fn to_sql(&self) -> String;
    fn execute(self, connection: &SqlitePool) -> Result<TExecuteResult, DbError>;
}

#[derive(Debug)]
struct GetMtimeQuery {
    node_id: NodeId,
}

#[derive(Debug, FromRow)]
struct MtimeQueryRow {
    mtime: String,
}

impl DbQuery<Option<Time>> for GetMtimeQuery {
    fn to_sql(&self) -> String {
        match &self.node_id {
            NodeId::Attrname(name) => {
                format!(
                    r"
                    select mtime
                    from node
                    join alias on alias.node_id = node.id
                    where alias.alias = '{name}'
            ",
                )
            }

            NodeId::Path(path) => {
                let path = path.to_string_lossy();

                format!(
                    r"
                    select mtime from node where path = '{path}'
                ",
                )
            }
        }
    }

    fn execute(self, connection: &SqlitePool) -> Result<Option<Time>, DbError> {
        Runtime::new()
            .unwrap()
            .block_on(sqlx::query_as(&self.to_sql()).fetch_optional(connection))
            .map_err(DbError::query_error_with(self))?
            .and_then(|row: MtimeQueryRow| Time::from_str(&row.mtime).ok())
            .map(Ok)
            .transpose()
    }
}

/// # Errors
/// TODO
pub fn get_mtime(node_id: NodeId) -> Result<Option<Time>, DbError> {
    let connection = get_connection()?;

    GetMtimeQuery { node_id }.execute(&connection)
}

#[derive(Debug, FromRow)]
struct ReturningIdInsertRow {
    id: i64,
}

struct SetNodeMtimeQuery<'a> {
    path: &'a AbsolutePathBuf,
    mtime: &'a Time,
}

impl DbQuery<i64> for SetNodeMtimeQuery<'_> {
    fn to_sql(&self) -> String {
        let Self { path, mtime } = self;

        let mtime = mtime.to_rfc3339();

        format!(
            r"
INSERT INTO node (path, mtime)
    VALUES ('{path}', '{mtime}')
ON CONFLICT (path)
    DO UPDATE SET
        mtime ='{mtime}'
    RETURNING
        id
            "
        )
    }

    fn execute(self, connection: &SqlitePool) -> Result<i64, DbError> {
        Runtime::new()
            .unwrap()
            .block_on(sqlx::query_as(&self.to_sql()).fetch_one(connection))
            .map_err(DbError::query_error_with(self))
            .map(|row: ReturningIdInsertRow| row.id)
    }
}

/// # Errors
/// TODO
pub fn set_node_mtime(path: &AbsolutePathBuf, mtime: &Time) -> Result<i64, DbError> {
    let connection = get_connection()?;

    SetNodeMtimeQuery { path, mtime }.execute(&connection)
}

struct InsertAliasQuery<'a> {
    alias: &'a Attrname,
    id: i64,
}

impl DbQuery<()> for InsertAliasQuery<'_> {
    fn to_sql(&self) -> String {
        let Self { alias, id } = self;

        format!(
            r"
INSERT INTO alias (alias, node_id)
    VALUES ('{alias}', {id})
ON CONFLICT (alias)
    DO NOTHING
            ",
        )
    }

    fn execute(self, connection: &SqlitePool) -> Result<(), DbError> {
        let _: Option<()> = Runtime::new()
            .unwrap()
            .block_on(sqlx::query_as(&self.to_sql()).fetch_optional(connection))
            .map_err(DbError::query_error_with(self))?;

        Ok(())
    }
}

/// # Errors
/// TODO
pub fn set_alias_mtime(
    alias: &Attrname,
    path: &AbsolutePathBuf,
    mtime: &Time,
) -> Result<(), DbError> {
    let connection = get_connection()?;

    let id = SetNodeMtimeQuery { path, mtime }.execute(&connection)?;

    InsertAliasQuery { alias, id }.execute(&connection)
}

#[derive(Debug, FromRow)]
struct StoreQueryRow {
    store_path: String,
}

struct GetStoreBinPathQuery<'a> {
    alias: &'a Attrname,
}

impl DbQuery<Option<NixStorePath>> for GetStoreBinPathQuery<'_> {
    fn to_sql(&self) -> String {
        let Self { alias } = self;

        format!(
            r"
SELECT
    store_path
FROM
    store
    JOIN alias ON alias.id = store.alias_id
WHERE
    alias.alias = '{alias}'
            ",
        )
    }

    fn execute(self, connection: &SqlitePool) -> Result<Option<NixStorePath>, DbError> {
        Runtime::new()
            .unwrap()
            .block_on(sqlx::query_as(&self.to_sql()).fetch_optional(connection))
            .map_err(DbError::query_error_with(self))?
            .map(|row: StoreQueryRow| row.store_path)
            .map(NixStorePath::new)
            .map(Ok)
            .transpose()
    }
}

/// # Errors
/// TODO
pub fn get_store_bin_path(alias: &Attrname) -> Result<Option<NixStorePath>, DbError> {
    let connection = get_connection()?;

    GetStoreBinPathQuery { alias }.execute(&connection)
}

struct WriteStoreQuery<'a> {
    alias: &'a Attrname,
    store_path: NixStorePath,
}

impl DbQuery<()> for WriteStoreQuery<'_> {
    fn to_sql(&self) -> String {
        let Self { alias, store_path } = self;

        let store_path = store_path.clone().into_string();

        format!(
            r"
INSERT INTO store (alias_id, store_path)
    VALUES ((
            SELECT
                id
            FROM
                alias
            WHERE
                alias = '{alias}'),
            '{store_path}')
ON CONFLICT(alias_id) 
DO UPDATE SET store_path = excluded.store_path;
            ",
        )
    }

    fn execute(self, connection: &SqlitePool) -> Result<(), DbError> {
        Runtime::new()
            .unwrap()
            .block_on(sqlx::query_as(&self.to_sql()).fetch_optional(connection))
            .map_err(DbError::query_error_with(self))
            .map(|_: Option<()>| ())
    }
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
            DbError::Connect(_) => value.into(),
            DbError::Query(e) => {
                if let Some(sqlx::error::ErrorKind::NotNullViolation) =
                    e.0.as_database_error()
                        .map(sqlx::error::DatabaseError::kind)
                {
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
pub fn write_store(alias: &Attrname, store_path: NixStorePath) -> WriteStoreResult {
    let connection = get_connection()?;

    Ok(WriteStoreQuery { alias, store_path }.execute(&connection)?)
}
