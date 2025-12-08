use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, sqlite::SqlitePoolOptions};

pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbRoute {
    pub id: i64,
    pub path: String,
    pub upstream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbApiKey {
    pub id: i64,
    pub name: String,
    pub key_hash: String,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS routes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            upstream TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            key_hash TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn get_routes(&self) -> Result<Vec<DbRoute>, sqlx::Error> {
        sqlx::query_as::<_, DbRoute>("SELECT id, path, upstream FROM routes")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn add_route(&self, path: &str, upstream: &str) -> Result<DbRoute, sqlx::Error> {
        sqlx::query_as::<_, DbRoute>(
            "INSERT INTO routes(path, upstream) VALUES (?, ?) RETURNING id, path, upstream",
        )
        .bind(path)
        .bind(upstream)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_route(&self, path: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM routes WHERE path = ?")
            .bind(path)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_api_keys(&self) -> Result<Vec<DbApiKey>, sqlx::Error> {
        sqlx::query_as::<_, DbApiKey>("SELECT id, name, key_hash FROM api_keys")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn add_api_key(&self, name: &str, key_hash: &str) -> Result<DbApiKey, sqlx::Error> {
        sqlx::query_as::<_, DbApiKey>(
            "INSERT INTO api_keys (name, key_hash) VALUES (?, ?) RETURNING id, name, key_hash",
        )
        .bind(name)
        .bind(key_hash)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_api_key(&self, name: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM api_keys WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
