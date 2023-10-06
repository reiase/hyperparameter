#[cfg(test)]
#[macro_use]
extern crate proptest;

pub mod entry;
pub mod storage;

pub mod api;
pub mod ffi;
pub mod xxh;

pub use crate::api::frozen_global_params;
pub use crate::api::ParamScope;
pub use crate::api::ParamScopeOps;