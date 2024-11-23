use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use redis::AsyncCommands;
use tokio::sync::RwLock;
use crate::{Cache, Cacheable};

/// Cache using redis.
/// 
/// It uses [`redis::Client`] to connect to redis.
/// Feature `redis` must be enabled.
/// 
/// A custom prefix should be specified. It will be used as the prefix of the redis key.
/// 
/// [`RedisCache`] implements [`Cache`]. See [`Cache`] for more details.
/// 
/// ## Example
/// 
/// ```rust,ignore
/// let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
/// 
/// // `aaa` is the prefix
/// let prefix = "aaa";
/// let cache = RedisCache::new(client, prefix).await.unwrap(); 
/// 
/// cache.set("a", 1).await.unwrap();
/// assert_eq!(cache.get::<u8>("a").await.unwrap().unwrap(), 1);
/// 
/// // Redis Data:
/// // aaa:a -> {1}
/// ```
#[derive(Debug)]
pub struct RedisCache<K: AsRef<str> + Send + Sync> {
    __marker: PhantomData<K>,
    inner: Arc<RwLock<Inner>>,
}

impl<K> RedisCache<K>
where
    K: AsRef<str> + Send + Sync
{
    pub async fn new<S: ToString>(client: redis::Client, prefix: S) -> anyhow::Result<Self> {
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            __marker: Default::default(),
            inner: Arc::new(RwLock::new(Inner {
                prefix: prefix.to_string(),
                client,
                conn,
            }))
        })
    }
}

impl<K> Clone for RedisCache<K>
where
    K: AsRef<str> + Send + Sync
{
    fn clone(&self) -> Self {
        Self {
            __marker: Default::default(),
            inner: self.inner.clone(),
        }
    }
}

#[async_trait::async_trait]
impl<K> Cache for RedisCache<K>
where
    K: AsRef<str> + Send + Sync,
{
    type Key = K;

    async fn get<T: Cacheable + Send + Sync>(&self, key: Self::Key) -> anyhow::Result<Option<T>> {
        let data: Option<String> = {
            let mut inner = self.inner.write().await;
            let key = inner.redis_key(key.as_ref());
            inner.conn.get(key).await?
        };

        let ret = data.as_deref()
            .map(T::from_hex)
            .transpose()?;

        Ok(ret)
    }

    async fn set<T: Cacheable + Send + Sync>(&self, key: Self::Key, value: T) -> anyhow::Result<()> {
        let s = value.to_hex();

        let mut inner = self.inner.write().await;
        let key = inner.redis_key(key.as_ref());
        inner.conn.set(key, s).await?;

        Ok(())
    }

    async fn delete(&self, key: Self::Key) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;
        let key = inner.redis_key(key.as_ref());
        inner.conn.del(key).await?;

        Ok(())
    }

    async fn len(&self) -> anyhow::Result<usize> {
        Ok(0)
    }
}

struct Inner {
    prefix: String,
    client: redis::Client,
    conn: redis::aio::MultiplexedConnection,
}

impl Debug for Inner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache.Inner")
            .field("client", &self.client)
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl Inner {
    pub fn redis_key(&self, key: &str) -> String {
        format!("{}:{}", &self.prefix, key)
    }
}

#[cfg(test)]
mod tests {
    use redis::Client;
    use super::*;

    #[test]
    fn it_works() {}

    #[tokio::test]
    async fn test_redis_cache() -> anyhow::Result<()> {
        let client = Client::open("redis://127.0.0.1:6379/")?;
        let cache = RedisCache::new(client, "aaa").await?;

        assert_eq!(cache.get::<u8>("none").await.unwrap(), None);
        cache.set("a", String::from("aaaaaa")).await?;
        assert_eq!(cache.get::<String>("a").await.unwrap().unwrap(), String::from("aaaaaa"));
        cache.set("a", String::from("aaaaaaa")).await?;
        assert_eq!(cache.get::<String>("a").await.unwrap().unwrap(), String::from("aaaaaaa"));

        cache.set("b", String::from("bbbbbb")).await?;
        assert_eq!(cache.get::<String>("b").await.unwrap().unwrap(), String::from("bbbbbb"));
        cache.set("c", 1).await?;
        assert_eq!(cache.get::<usize>("c").await.unwrap().unwrap(), 1);
        Ok(())
    }
}
