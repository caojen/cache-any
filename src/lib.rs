#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "redis", allow(dependency_on_unit_never_type_fallback))]

//! A cache library for Rust.
//! 
//! This library provides a trait [`Cache`] and some
//! implementations of it.
//! 
//! It defines the basic operations of a cache, for
//! example, [`caches::Cache::get`], [`caches::Cache::set`].
//! All functions are async, because we may use async storage backends.
//! All caches are key-value based.
//! 
//! By default, it provides a simple memory cache as example. See [`caches::MemoryCache`].
//! Other caches are available in below features:
//! 
//! * `redis`: Use redis as storage backend. See [`caches::RedisCache`].
//! 
//! ## Concepts
//! 
//! * **Key**: The key of a cache is [`Cache::Key`]. It is specified by the cache implementation. Usually it is a string-like type.
//! * **Value**: The value of a cache is a [`Cacheable`] value.
//! 
//! [`Cacheable`] is a trait that describes how to convert a `value` to bytes and vice versa.
//! 
//! A cache can store any value that implements [`Cacheable`].
//! That is, you can store usize and string (or any other types) at the same time.
//! But you need to know the exact type when you retrieve the value.
//! 
//! ## Basic Usage
//! 
//! We use [`caches::MemoryCache`] as example.
//! 
//! ```rust
//! let cache = MemoryCache::default();
//! 
//! // The cache is empty, so `get` returns None.
//! assert!(cache.get::<()>("non-existent-key").await.unwrap().is_none());
//! 
//! // [SET a -> 1]
//! cache.set("a", 1).await.unwrap();
//! 
//! // [GET a] -> Some(1)
//! let a_value: u8 = cache.get("a").await.unwrap().unwrap();
//! assert_eq!(a_value, 1);
//! 
//! // you can do type casting, using u16 instead of u8 as an example.
//! let a_value: u16 = cache.get("a").await.unwrap().unwrap();
//! assert_eq!(a_value, 1);
//! 
//! // you can also store [`String`] in the same cache:
//! cache.set("b", String::from("hello")).await.unwrap();
//! let b_value: String = cache.get("b").await.unwrap().unwrap();
//! assert_eq!(b_value, String::from("hello"));
//! ```
//! 
//! ## Extend Cacheable
//! 
//! You can extend [`Cacheable`] for your own types.
//! For example, you can define a struct and implement [`Cacheable`] for it.
//! 
//! ```rust
//! #[derive(serde::Serialize, serde::Deserialize)] // for json
//! struct MyStruct {
//!     a: u8,
//!     b: String,
//! }
//! ```
//! 
//! In this case, we use `serde_json` to convert the struct to bytes and vice versa.
//! 
//! ```rust,ignore
//! impl Cacheable for MyStruct {
//!     fn to_bytes(&self) -> Vec<u8> {
//!         serde_json::to_vec(self).unwrap()
//!     }
//!
//!     fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
//!         let ret = serde_json::from_slice(bytes)?;
//!         Ok(ret)
//!     }
//! }
//! ```
//! 
//! Then you can store `MyStruct` in the cache:
//! 
//! ```rust
//! cache.set("my-struct", MyStruct { a: 1, b: String::from("hello") }).await.unwrap();
//! ```
//!
//! ## Contributing
//! 
//! Any contributions are welcome.
//! 
//! If you find any useful cache implementation,
//! feel free to open an issue or a pull request at [Github](https://github.com/caojen/cache-any).
//! 
//! If bugs are found, just file an issue at [Github](https://github.com/caojen/cache-any), and I will fix it **ASAP**.


mod cacheable;
pub use cacheable::*;

mod caches;
pub use caches::*;

#[test]
fn it_works() {
    println!("it works")
}
