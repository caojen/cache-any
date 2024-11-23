use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::sync::RwLock;
use crate::Cacheable;
use crate::Cache;

/// Cache using memory.
/// 
/// Data is stored in memory. However, this cache will serialize and deserialize data,
/// so it may not be so efficient.
/// 
/// [`MemoryCache`] implements [`Cache`]. See [`Cache`] for more details.
/// 
/// ## Example
/// 
/// ```rust
/// let cache = MemoryCache::default();
/// 
/// cache.set("a", 1).await.unwrap();
/// assert_eq!(cache.get::<u8>("a").await.unwrap().unwrap(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct MemoryCache {
    inner: Arc<RwLock<Inner>>,
}

impl Default for MemoryCache
{
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryCache
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
impl Cache for MemoryCache
{
    async fn get<T: Cacheable + Send + Sync>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let inner = self.inner.read().await;
        let ret = inner.get(key.as_bytes())
            .map(|val| val.as_slice())
            .map(T::from_bytes)
            .transpose()?;

        Ok(ret)
    }

    async fn set<T: Cacheable + Send + Sync>(&self, key: &str, value: T) -> anyhow::Result<()> {
        let bytes = value.to_bytes();

        let mut inner = self.inner.write().await;
        inner.insert(key.as_bytes().to_vec(), bytes);

        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.write().await;
        inner.remove(key.as_bytes());

        Ok(())
    }

    async fn len(&self) -> anyhow::Result<usize> {
        let inner = self.inner.read().await;
        Ok(inner.len())
    }
}

#[derive(Debug)]
struct Inner {
    map: HashMap<Vec<u8>, Vec<u8>>,
}

impl Deref for Inner
{
    type Target = HashMap<Vec<u8>, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Inner
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

        assert_eq!(cache.get::<()>("non-existent-key").await?, None);

        Ok(())
    }
}
