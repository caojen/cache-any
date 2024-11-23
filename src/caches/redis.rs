use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use redis::AsyncCommands;
use tokio::sync::RwLock;
use crate::{Cache, Cacheable};

/// Cache using redis.
/// 
/// It uses [`redis::Client`] to connect to redis.
/// Feature `redis` must be enabled.
/// 
/// A custom map should be specified. It will be used as the map of the redis key.
/// 
/// [`RedisCache`] implements [`Cache`]. See [`Cache`] for more details.
/// 
/// ## Example
/// 
/// ```rust,ignore
/// let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
/// 
/// // `aaa` is the hash map name
/// let map = "aaa";
/// let cache = RedisCache::new(client, map).await.unwrap(); 
/// 
/// cache.set("a", 1).await.unwrap();
/// assert_eq!(cache.get::<u8>("a").await.unwrap().unwrap(), 1);
/// 
/// // Redis Data ('aaa' is a redis hash map):
/// // aaa: a -> Encoded(1)
/// ```
#[derive(Debug, Clone)]
pub struct RedisCache {
    inner: Arc<RwLock<Inner>>,
}

impl RedisCache {
    pub async fn new<S: ToString>(client: redis::Client, map: S) -> anyhow::Result<Self> {
        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            inner: Arc::new(RwLock::new(Inner {
                map: Arc::new(map.to_string()),
                client,
                conn,
            }))
        })
    }
}

#[async_trait::async_trait]
impl Cache for RedisCache {
    async fn get<T: Cacheable + Send + Sync>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let val: Option<String> = {
            let mut inner = self.inner.write().await;
            let map = inner.map.clone();
            inner.conn.hget(map, key).await?
        };

        val.map(|val| T::from_hex(&val))
            .transpose()
    }

    async fn set<T: Cacheable + Send + Sync>(&self, key: &str, value: T) -> anyhow::Result<()> {
        let val = value.to_hex();
        let mut inner = self.inner.write().await;
        let map = inner.map.clone();
        inner.conn.hset(map, key, val).await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;
        let map = inner.map.clone();
        inner.conn.hdel(map, key).await?;

        Ok(())
    }

    async fn len(&self) -> anyhow::Result<usize> {
        let mut inner = self.inner.write().await;
        let map = inner.map.clone();
        let len: u64 = inner.conn.hlen(map).await?;

        Ok(len as usize)
    }
}

struct Inner {
    map: Arc<String>,
    client: redis::Client,
    conn: redis::aio::MultiplexedConnection,
}

impl Debug for Inner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache.Inner")
            .field("client", &self.client)
            .field("map", &self.map)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use redis::Client;
    use super::*;

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

        let cloned = cache.clone();
        assert_eq!(cloned.get::<u8>("none").await.unwrap(), None);
        cloned.set("a", String::from("aaaaaa")).await?;
        assert_eq!(cloned.get::<String>("a").await.unwrap().unwrap(), String::from("aaaaaa"));
        cloned.set("a", String::from("aaaaaaa")).await?;
        assert_eq!(cloned.get::<String>("a").await.unwrap().unwrap(), String::from("aaaaaaa"));

        cloned.set("b", String::from("bbbbbb")).await?;
        assert_eq!(cloned.get::<String>("b").await.unwrap().unwrap(), String::from("bbbbbb"));
        cloned.set("c", 1).await?;
        assert_eq!(cloned.get::<usize>("c").await.unwrap().unwrap(), 1);

        println!("{:?}", cache);
        println!("{:?}", cloned);
        println!("size = {}", cloned.len().await?);
        Ok(())
    }
}
