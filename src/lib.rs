pub unsafe extern "C" fn test() {}

pub mod entry;
pub mod tls_storage;
pub mod tree_storage;

#[cfg(test)]
extern crate rspec;
