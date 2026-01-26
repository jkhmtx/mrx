use std::env;
use std::env::VarError;
use std::str::FromStr as _;

use mrx_utils::Attrname;
use mrx_utils::fs::AbsolutePathBuf;
use mrx_utils::graph::NodeId;
use mrx_utils::nix_store_path::NixStorePath;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::types::chrono::NaiveDateTime;
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
    Query(sqlx::Error),
}

fn get_connection() -> Result<SqlitePool, ConnectError> {
    let database_url = env::var("DATABASE_URL")?;
    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let rt = Runtime::new().unwrap();

    Ok(rt.block_on(SqlitePool::connect_with(options))?)
}

trait DbQuery<TExecuteResult> {
    fn into_sql(self) -> String;
    fn execute(self, connection: &SqlitePool) -> Result<TExecuteResult, DbError>;
}

#[derive(Debug)]
struct GetMtimeQuery {
    node_id: NodeId,
}

#[derive(Debug, FromRow)]
struct MtimeQueryRow {
    mtime: NaiveDateTime,
}

impl DbQuery<Option<Time>> for GetMtimeQuery {
    fn into_sql(self) -> String {
        match self.node_id {
            NodeId::Attrname(name) => {
                format!(
                    r"
                    select mtime
                    from node
                    join alias on alias.node_id = node.id
                    where alias.alias = {name}
            ",
                )
            }

            NodeId::Path(path) => {
                let path = path.to_string_lossy();

                format!(
                    r"
                    select mtime from node where path = {path}
                ",
                )
            }
        }
    }

    fn execute(self, connection: &SqlitePool) -> Result<Option<Time>, DbError> {
        let rt = Runtime::new().unwrap();

        let sql = self.into_sql();
        let mtime: Option<MtimeQueryRow> = rt
            .block_on(sqlx::query_as(&sql).fetch_optional(connection))
            .map_err(DbError::Query)?;

        Ok(mtime.map(|mtime| mtime.mtime.and_utc()))
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
    fn into_sql(self) -> String {
        let Self { path, mtime } = self;

        format!(
            r"
INSERT INTO node (path, mtime)
    VALUES ({path}, {mtime})
ON CONFLICT (path)
    DO UPDATE SET
        mtime = ?2
    RETURNING
        id
            ",
        )
    }

    fn execute(self, connection: &SqlitePool) -> Result<i64, DbError> {
        let rt = Runtime::new().unwrap();

        let sql = self.into_sql();
        let returning: ReturningIdInsertRow = rt
            .block_on(sqlx::query_as(&sql).fetch_one(connection))
            .map_err(DbError::Query)?;

        Ok(returning.id)
    }
}

/// # Errors
/// TODO
pub fn set_node_mtime(path: &AbsolutePathBuf, mtime: &Time) -> Result<i64, DbError> {
    let connection = get_connection()?;

    SetNodeMtimeQuery { path, mtime }.execute(&connection)
}

/// # Errors
/// TODO
pub async fn set_alias_mtime(
    alias: &Attrname,
    path: &AbsolutePathBuf,
    mtime: &Time,
) -> Result<(), DbError> {
    let alias = alias.to_string();

    let connection = get_connection()?;

    let id = set_node_mtime(path, mtime)?;

    sqlx::query!(
        r#"
INSERT INTO alias (alias, node_id)
    VALUES (?1, ?2)
ON CONFLICT (alias)
    DO NOTHING
            "#,
        alias,
        id
    )
    .fetch_optional(&connection)
    .await
    .map_err(DbError::Query)?;

    Ok(())
}

#[derive(Debug, FromRow)]
struct StoreQueryRow {
    store_path: String,
}

/// # Errors
/// TODO
pub async fn get_store_bin_path(alias: &Attrname) -> Result<Option<NixStorePath>, DbError> {
    let alias = alias.to_string();

    let connection = get_connection()?;

    let row = sqlx::query_as!(
        StoreQueryRow,
        r#"
SELECT
    store_path
FROM
    store
    JOIN alias ON alias.id = store.alias_id
WHERE
    alias.alias = ?
            "#,
        alias
    )
    .fetch_optional(&connection)
    .await
    .map_err(DbError::Query)?;

    Ok(row.map(|row| row.store_path).map(NixStorePath::new))
}

/// # Errors
/// TODO
pub async fn write_store(alias: Attrname, store_path: NixStorePath) -> Result<(), DbError> {
    let path = store_path.into_string();
    let alias = alias.into_downcast();

    let connection = get_connection()?;

    sqlx::query!(
        r#"
INSERT INTO store (alias_id, store_path)
    VALUES ((
            SELECT
                id
            FROM
                alias
            WHERE
                alias = ?1),
            ?2)
ON CONFLICT(alias_id) 
DO UPDATE SET store_path = excluded.store_path;
            "#,
        alias,
        path
    )
    .fetch_optional(&connection)
    .await
    .map_err(DbError::Query)?;

    Ok(())
}
