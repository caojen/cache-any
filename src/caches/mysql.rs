use std::sync::Arc;
use crate::{Cache, Cacheable};

#[derive(Debug, Clone)]
pub struct MysqlCache {
    inner: Arc<Inner>,
}

#[async_trait::async_trait]
impl Cache for MysqlCache {
    type Key = String;

    async fn get<T: Cacheable + Send + Sync>(&self, key: Self::Key) -> anyhow::Result<Option<T>> {
        let sql = format!(r#"
            SELECT {}
            FROM {}
            WHERE {} = ?
            LIMIT 1
        "#, &self.inner.value_field, &self.inner.table, &self.inner.key_field);

        let value: Option<(String,)> = sqlx::query_as(&sql)
            .bind(&key)
            .fetch_optional(&self.inner.pool)
            .await?;

        let result = value.map(|value| value.0)
            .map(String::as_str)
            .map(T::from_hex)
            .transpose()?;

        Ok(result)
    }

    async fn set<T: Cacheable + Send + Sync>(&self, key: Self::Key, value: T) -> anyhow::Result<()> {
        let value = value.to_hex();

        let sql = format!(r#"
            INSERT INTO {}
            SET {} = ?
            WHERE {} = ?
            ON DUPLICATE KEY UPDATE {} = ?
        "#,
            &self.inner.table,
            &self.inner.value_field,
            &self.inner.key_field,
            &self.inner.value_field,
        );

        sqlx::query(&sql)
            .bind(&value)
            .bind(&key)
            .bind(&value)
            .execute(&self.inner.pool)
            .await?;

        Ok(())
    }

    async fn delete(&self, key: Self::Key) -> anyhow::Result<()> {
        let sql = format!(r#"
            DELETE FROM {}
            WHERE {} = ?
        "#, &self.inner.table, &self.inner.key_field);

        sqlx::query(&sql)
            .bind(&key)
            .execute(&self.inner.pool)
            .await?;

        Ok(())
    }

    async fn len(&self) -> anyhow::Result<usize> {
        let sql = format!(r#"
            SELECT COUNT(*)
            FROM {}
        "#, &self.inner.table);

        let count: (i64,) = sqlx::query_as(&sql).fetch_optional(&self.inner.pool).await?.unwrap_or_default();

        Ok(count.0 as usize)
    }
}

#[derive(Debug, Clone)]
pub struct MysqlCacheBuilder {
    key_field: String,
    value_field: String,
    table: String,
    pool: sqlx::MySqlPool,
}

impl MysqlCacheBuilder {
    pub fn new(pool: sqlx::MySqlPool) -> Self {
        Self {
            key_field: String::from("key"),
            value_field: String::from("val"),
            table: String::from("cache"),
            pool,
        }
    }

    pub fn key_field<S: ToString>(mut self, key: S) -> Self {
        self.key_field = key.to_string();
        self
    }

    pub fn value_field<S: ToString>(mut self, value: S) -> Self {
        self.value_field = value.to_string();
        self
    }

    pub fn table<S: ToString>(mut self, table: S) -> Self {
        self.table = table.to_string();
        self
    }

    pub fn finish(self) -> MysqlCache {
        MysqlCache {
            inner: Arc::new(Inner {
                key_field: self.key_field,
                value_field: self.value_field,
                table: self.table,
                pool: self.pool,
            })
        }
    }
}

#[derive(Debug)]
struct Inner {
    key_field: String,
    value_field: String,
    table: String,
    pool: sqlx::MySqlPool,
}
