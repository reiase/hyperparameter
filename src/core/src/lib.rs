#[cfg(test)]
#[macro_use]
extern crate proptest;

mod storage;
mod value;

mod api;
mod cfg;
mod ffi;
mod xxh;

#[cfg(feature = "tokio-task-local")]
pub use crate::api::bind;
pub use crate::api::frozen;
pub use crate::api::ParamScope;
pub use crate::api::ParamScopeOps;
pub use crate::cfg::AsParamScope;
#[cfg(feature = "tokio-task-local")]
pub use crate::storage::storage_scope;
pub use crate::storage::with_current_storage;
pub use crate::storage::GetOrElse;
pub use crate::storage::THREAD_STORAGE;
pub use crate::value::Value;
pub use crate::xxh::xxhash;
pub use crate::xxh::XXHashable;
pub use const_str;
pub use xxhash_rust;

// Re-export procedural macros
pub use hyperparameter_macros::get_param;
pub use hyperparameter_macros::with_params;

#[cfg(feature = "clap")]
mod cli;
#[cfg(feature = "clap")]
pub use crate::cli::generate_params_help;
#[cfg(feature = "clap")]
pub use crate::cli::PARAMS;
