#[cfg(test)]
#[macro_use]
extern crate proptest;

mod storage;
mod value;

mod api;
mod cfg;
mod cli;
mod ffi;
mod xxh;

pub use crate::api::frozen;
pub use crate::api::ParamScope;
pub use crate::api::ParamScopeOps;
pub use crate::cfg::AsParamScope;
pub use crate::cli::generate_params_help;
pub use crate::cli::PARAMS;
pub use crate::storage::GetOrElse;
pub use crate::storage::THREAD_STORAGE;
pub use crate::value::Value;
pub use crate::xxh::xxhash;
pub use crate::xxh::XXHashable;
