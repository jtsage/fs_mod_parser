#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

pub mod maps;
pub mod mod_basic;
pub mod shared;

pub use mod_basic::parse_to_json as parse_basic_mod;
