
/// [`Cacheable`] trait.
///
/// It is used to convert [`Cacheable`] to bytes and vice versa.
pub trait Cacheable {
    /// Error type when [`Cacheable`] is converted from bytes.
    type Error: std::error::Error;

    /// Convert [`Cacheable`] to bytes.
    fn to_bytes(&self) -> Vec<u8>;

    /// Convert bytes to [`Cacheable`].
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
