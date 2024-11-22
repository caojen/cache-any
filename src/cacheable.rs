use std::fmt::Debug;

/// [`Cacheable`] trait.
///
/// It is used to convert [`Cacheable`] to bytes and vice versa.
pub trait Cacheable: Debug {
    /// Convert [`Cacheable`] to bytes.
    fn to_bytes(&self) -> Vec<u8>;

    /// Convert bytes to [`Cacheable`].
    fn from_bytes(bytes: &[u8]) -> crate::Result<Self>
    where
        Self: Sized;

    fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    fn from_hex(hex: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let bytes = hex::decode(hex)?;
        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> { Ok(()) }
}
