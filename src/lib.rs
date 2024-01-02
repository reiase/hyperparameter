#[cfg(test)]
#[macro_use]
extern crate proptest;

pub extern crate const_str;
pub extern crate xxhash_rust;

pub use crate::api::frozen;
pub use crate::api::ParamScope;
pub use crate::api::ParamScopeOps;
pub use crate::cfg::AsParamScope;
pub use crate::storage::GetOrElse;
pub use crate::storage::THREAD_STORAGE;
pub use crate::value::Value;
pub use crate::xxh::XXHashable;

pub mod storage;
pub mod value;

pub mod api;
pub mod cfg;
pub mod ffi;
pub mod xxh;
