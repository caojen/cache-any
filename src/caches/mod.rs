mod memory;
pub use memory::*;

#[cfg(feature = "redis")]
mod redis;
#[cfg(feature = "redis")]
pub use redis::*;

use std::hash::Hash;
use std::fmt::Debug;
use crate::Cacheable;

#[async_trait::async_trait]
pub trait Cache: Clone {
    type Key;

    async fn get<T: Cacheable + Send + Sync>(&self, key: Self::Key) -> anyhow::Result<Option<T>>;
    async fn set<T: Cacheable + Send + Sync>(&self, key: Self::Key, value: T) -> anyhow::Result<()>;
    async fn delete(&self, key: Self::Key) -> anyhow::Result<()>;
    async fn len(&self) -> anyhow::Result<usize>;
}
