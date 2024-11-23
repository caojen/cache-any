use cache_any::{Cache, MemoryCache};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cache = MemoryCache::with_capacity(1024);

    // init: cache size is 0
    assert_eq!(cache.len().await?, 0);

    cache.set("username", String::from("Jack")).await?;
    cache.set("user_id", 13).await?;

    let username: Option<String> = cache.get("username").await?;
    assert_eq!(username, Some(String::from("Jack")));
    let user_id: Option<usize> = cache.get("user_id").await?;
    assert_eq!(user_id, Some(13));

    // delete key
    cache.delete("username").await?;
    let username: Option<String> = cache.get("username").await?;
    assert!(username.is_none());

    let len = cache.len().await?;
    assert_eq!(len, 1);

    Ok(())
}
