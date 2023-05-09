// #![feature(local_key_cell_methods)]
// #![feature(let_chains)]

pub mod entry;
pub mod storage;

pub mod ext;
pub mod ffi;
pub mod xxh;

#[cfg(test)]
extern crate rspec;
