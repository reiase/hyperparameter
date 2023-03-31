#![feature(local_key_cell_methods)]
#![feature(let_chains)]

pub mod entry;
pub mod storage;

pub mod ext;

#[cfg(test)]
extern crate rspec;
