#[cfg(test)]
#[macro_use]
extern crate proptest;

pub use crate::api::frozen_global_params;
pub use crate::api::ParamScope;
pub use crate::api::ParamScopeOps;
pub use crate::storage::GetOrElse;
pub use crate::storage::THREAD_STORAGE;
pub use crate::value::Value;
pub use crate::xxh::XXHashable;

pub mod storage;
pub mod value;

pub mod api;
pub mod debug;
pub mod ffi;
pub mod xxh;
