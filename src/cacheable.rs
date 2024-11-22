use std::fmt::Debug;
use std::io::Cursor;
use std::sync::Arc;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

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

impl<T: Cacheable> Cacheable for Arc<T> {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_ref().to_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Arc::new(T::from_bytes(bytes)?))
    }
}

impl Cacheable for String {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::from_utf8(bytes.to_vec())?)
    }
}

macro_rules! impl_numeric {
    ($ty: ty) => {
        impl Cacheable for $ty {
            fn to_bytes(&self) -> Vec<u8> {
                let num = *self as u128;
                let mut wtr = Vec::with_capacity(16);
                wtr.write_u128::<BigEndian>(num).unwrap();

                wtr
            }

            fn from_bytes(bytes: &[u8]) -> crate::Result<Self>
            where
                Self: Sized
            {
                let mut rdr = Cursor::new(bytes);
                let num = rdr.read_u128::<BigEndian>().unwrap();

                Ok(num as $ty)
            }
        }
    };
    ($($ty: ty),+ $(,)?) => {
        $(
            impl_numeric!($ty);
        )*
    };
}

impl_numeric!(
    u128, i128,
    u64, i64,
    u32, i32,
    u16, i16,
    u8, i8,
    usize, isize,
);

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> { Ok(()) }

    #[test]
    fn test_numeric() -> anyhow::Result<()> {
        macro_rules! test {
            ($ty: ty) => {
                for _ in 0..1024 {
                    let num: $ty = rand::thread_rng().gen();
                    let v = Cacheable::to_bytes(&num);
                    let d: $ty = Cacheable::from_bytes(&v).unwrap();
                    assert_eq!(num, d);
                }
            };
            ($($ty: ty),+ $(,)?) => {
                $(test!($ty);)+
            };
        }

        test!(
            u128, i128,
            u64, i64,
            u32, i32,
            u16, i16,
            u8, i8,
            usize, isize,
        );

        macro_rules! test_arc {
            ($ty: ty) => {
                for _ in 0..1024 {
                    let num: $ty = rand::thread_rng().gen();
                    let num = Arc::new(num);
                    let v = Cacheable::to_bytes(&num);
                    let arc_d: Arc<$ty> = Cacheable::from_bytes(&v).unwrap();
                    let d = Cacheable::from_bytes(&v).unwrap();
                    assert_eq!(num, arc_d);
                    assert_eq!(d, arc_d);
                }
            };
            ($($ty: ty),+ $(,)?) => {
                $(test_arc!($ty);)+
            };
        }

        Ok(())
    }
}
