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

async fn get_connection() -> Result<SqlitePool, ConnectError> {
    let database_url = env::var("DATABASE_URL")?;
    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    Ok(SqlitePool::connect_with(options).await?)
}

#[derive(Debug, FromRow)]
struct MtimeQueryRow {
    mtime: NaiveDateTime,
}

/// # Errors
/// TODO
pub async fn get_mtime(id: NodeId) -> Result<Option<Time>, DbError> {
    let connection = get_connection().await?;

    let mtime = match &id {
        NodeId::Attrname(name) => {
            let name = name.as_str();
            sqlx::query_as!(
                MtimeQueryRow,
                r#"
                    select mtime
                    from node
                    join alias on alias.node_id = node.id
                    where alias.alias = ?
            "#,
                name
            )
            .fetch_optional(&connection)
            .await
        }

        NodeId::Path(path) => {
            let path = path.to_string_lossy();

            sqlx::query_as!(
                MtimeQueryRow,
                r#"
                    select mtime from node where path = ?
                "#,
                path
            )
            .fetch_optional(&connection)
            .await
        }
    }
    .map_err(DbError::Query)?;

    Ok(mtime.map(|mtime| mtime.mtime.and_utc()))
}

#[derive(Debug, FromRow)]
struct ReturningIdInsertRow {
    id: i64,
}

/// # Errors
/// TODO
pub async fn set_node_mtime(path: &AbsolutePathBuf, mtime: &Time) -> Result<i64, DbError> {
    let path = path.to_string();
    let connection = get_connection().await?;

    let returning = sqlx::query_as!(
        ReturningIdInsertRow,
        r#"
INSERT INTO node (path, mtime)
    VALUES (?1, ?2)
ON CONFLICT (path)
    DO UPDATE SET
        mtime = ?2
    RETURNING
        id
            "#,
        path,
        mtime
    )
    .fetch_one(&connection)
    .await
    .map_err(DbError::Query)?;

    Ok(returning.id)
}

/// # Errors
/// TODO
pub async fn set_alias_mtime(
    alias: &Attrname,
    path: &AbsolutePathBuf,
    mtime: &Time,
) -> Result<(), DbError> {
    let alias = alias.to_string();

    let connection = get_connection().await?;

    let id = set_node_mtime(path, mtime).await?;

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

    let connection = get_connection().await?;

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

    let connection = get_connection().await?;

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
