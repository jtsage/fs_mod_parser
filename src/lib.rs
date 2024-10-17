#![doc = include_str!("../README.md")]


pub mod maps;
pub mod mod_basic;
pub mod shared;

pub use mod_basic::parse_to_json as parse_basic_mod;
