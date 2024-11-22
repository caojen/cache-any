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

#[async_trait::async_trait]
impl<K> Cache for MemoryCache<K>
where
    K: Hash + Debug + Clone + Eq + PartialEq + Send + Sync,
{
    type Key = K;

    async fn get<T: Cacheable + Send + Sync>(&self, key: Self::Key) -> crate::Result<Option<T>> {
        let inner = self.inner.read().await;
        let ret = inner.get(&key)
            .map(|val| val.as_slice())
            .map(T::from_bytes)
            .transpose()?;

        Ok(ret)
    }

    async fn set<T: Cacheable + Send + Sync>(&self, key: Self::Key, value: T) -> crate::Result<()> {
        let bytes = value.to_bytes();

        let mut inner = self.inner.write().await;
        inner.insert(key, bytes);

        Ok(())
    }

    async fn delete(&self, key: Self::Key) -> crate::Result<()> {
        let mut inner = self.inner.write().await;
        inner.remove(&key);

        Ok(())
    }

    async fn len(&self) -> crate::Result<usize> {
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
