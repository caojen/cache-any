use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;
use tokio::sync::RwLock;
use crate::Cacheable;
use crate::Cache;
#[derive(Debug, Clone)]
pub struct MemoryCache<K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync> {
    inner: Arc<RwLock<Inner<K>>>,
}

impl<K> Default for MemoryCache<K>
where K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K> MemoryCache<K>
where K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync
{
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                map: HashMap::with_capacity(cap),
            }))
        }
    }
}

#[async_trait::async_trait]
impl<K> Cache for MemoryCache<K>
where
    K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync,
{
    type Key = K;

    async fn get<T: Cacheable + Send + Sync>(&self, key: Self::Key) -> anyhow::Result<Option<T>> {
        let inner = self.inner.read().await;
        let ret = inner.get(&key)
            .map(|val| val.as_slice())
            .map(T::from_bytes)
            .transpose()?;

        Ok(ret)
    }

    async fn set<T: Cacheable + Send + Sync>(&self, key: Self::Key, value: T) -> anyhow::Result<()> {
        let bytes = value.to_bytes();

        let mut inner = self.inner.write().await;
        inner.insert(key, bytes);

        Ok(())
    }

    async fn delete(&self, key: Self::Key) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;
        inner.remove(&key);

        Ok(())
    }

    async fn len(&self) -> anyhow::Result<usize> {
        let inner = self.inner.read().await;
        Ok(inner.len())
    }
}

#[derive(Debug)]
struct Inner<K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync> {
    map: HashMap<K, Vec<u8>>,
}

impl<K> Deref for Inner<K>
where
    K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync,
{
    type Target = HashMap<K, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<K> DerefMut for Inner<K>
where
    K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map   
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache() -> anyhow::Result<()> {
        let cache = MemoryCache::default();

        let len = cache.len().await?;
        assert_eq!(len, 0);

        cache.set("a", 1).await?;
        cache.set("b", vec![0u8, 1u8, 2u8, 4u8]).await?;
        cache.set("c", String::from("ccc")).await?;

        assert_eq!(cache.get::<u8>("a").await?.unwrap(), 1u8);
        assert_eq!(cache.get::<Vec<u8>>("b").await?.unwrap(), vec![0u8, 1u8, 2u8, 4u8]);
        assert_eq!(cache.get::<String>("c").await?.unwrap(), String::from("ccc"));
        assert_eq!(cache.get::<String>("d").await?, None);

        let nc = cache.clone();
        assert_eq!(cache.get::<u8>("a").await?.unwrap(), 1u8);
        assert_eq!(cache.get::<Vec<u8>>("b").await?.unwrap(), vec![0u8, 1u8, 2u8, 4u8]);
        assert_eq!(cache.get::<String>("c").await?.unwrap(), String::from("ccc"));
        assert_eq!(cache.get::<String>("d").await?, None);
        assert_eq!(nc.len().await.unwrap(), 3);

        Ok(())
    }
}
