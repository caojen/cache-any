#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "redis", allow(dependency_on_unit_never_type_fallback))]

mod cacheable;
pub use cacheable::*;

mod caches;
pub use caches::*;

#[test]
fn it_works() {
    println!("it works")
}
