mod memory;
pub use memory::*;

#[cfg(feature = "redis")]
mod redis;
#[cfg(feature = "redis")]
pub use redis::*;

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
pub use mysql::*;

use crate::Cacheable;

/// A cache trait.
/// 
/// It describes the basic operations of a cache.
/// All functions are async, because we may use async storage backends.
#[async_trait::async_trait]
#[allow(clippy::len_without_is_empty)]
pub trait Cache: Clone {
    type Key;

    async fn get<T: Cacheable + Send + Sync>(&self, key: Self::Key) -> anyhow::Result<Option<T>>;
    async fn set<T: Cacheable + Send + Sync>(&self, key: Self::Key, value: T) -> anyhow::Result<()>;
    async fn delete(&self, key: Self::Key) -> anyhow::Result<()>;
    async fn len(&self) -> anyhow::Result<usize>;
}
