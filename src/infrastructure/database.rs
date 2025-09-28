use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::str::FromStr;

pub async fn create_sqlite_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .read_only(false)
        .foreign_keys(true)
        .create_if_missing(true);

    SqlitePool::connect_with(opts).await
}
