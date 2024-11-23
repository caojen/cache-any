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

        let result = value.as_ref()
            .map(|value| &value.0)
            .map(|value| value.as_str())
            .map(T::from_hex)
            .transpose()?;

        Ok(result)
    }

    async fn set<T: Cacheable + Send + Sync>(&self, key: Self::Key, value: T) -> anyhow::Result<()> {
        let value = value.to_hex();

        let sql = format!(r#"
            INSERT INTO {} ({}, {})
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE {} = ?
        "#,
            &self.inner.table,
            &self.inner.key_field,
            &self.inner.value_field,
            &self.inner.value_field,
        );

        sqlx::query(&sql)
            .bind(&key)
            .bind(&value)
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
            key_field: String::from("name"),
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

#[cfg(test)]
mod tests {
    use sqlx::MySqlPool;
    use super::*;

    #[tokio::test]
    async fn test_mysql_cache_builder() -> anyhow::Result<()> {
        let pool = MySqlPool::connect("mysql://test:123456@127.0.0.1:3306/dev").await?;
        let cache = MysqlCacheBuilder::new(pool)
            .table("my_cache")
            .key_field("name")
            .value_field("val")
            .finish();

        assert_eq!(cache.inner.table, String::from("my_cache"));
        assert_eq!(cache.inner.key_field, String::from("name"));
        assert_eq!(cache.inner.value_field, String::from("val"));

        let cloned_cache = cache.clone();
        assert_eq!(cloned_cache.inner.table, String::from("my_cache"));
        assert_eq!(cloned_cache.inner.key_field, String::from("name"));
        assert_eq!(cloned_cache.inner.value_field, String::from("val"));

        println!("{:?}", cloned_cache);

        Ok(())
    }

    #[tokio::test]
    async fn test_mysql_cache() -> anyhow::Result<()> {
        // create user test@'%' identified by '123456';
        // create database dev;
        // grant all privileges on dev.* to test@'%';
        //
        // CREATE TABLE IF NOT EXISTS my_cache (
        //     name varchar(255) not null,
        //     val text not null,
        //     primary key (name)
        // );

        let pool = MySqlPool::connect("mysql://test:123456@127.0.0.1:3306/dev").await?;

        let cache = MysqlCacheBuilder::new(pool)
            .table("my_cache")
            .key_field("name")
            .value_field("val")
            .finish();

        cache.set(String::from("user_id"), 114514).await?;
        cache.set(String::from("username"), String::from("jack")).await?;

        let user_id: usize = cache.get(String::from("user_id")).await?.unwrap();
        let username: String = cache.get(String::from("username")).await?.unwrap();

        assert_eq!(user_id, 114514);
        assert_eq!(username, String::from("jack"));

        cache.delete(String::from("user_id")).await?;
        let user_id: Option<()> = cache.get(String::from("user_id")).await?;
        assert_eq!(user_id, None);

        let len = cache.len().await?;
        println!("len = {}", len);

        Ok(())
    }
}
