mod cacheable;
pub use cacheable::*;

mod caches;
pub use caches::*;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[test]
fn it_works() {
    println!("it works")
}
