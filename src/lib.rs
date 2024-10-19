#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

pub mod maps;
pub mod mod_basic;
pub mod savegame;
pub mod shared;

pub use savegame::parse_to_json as parse_savegame;
pub use mod_basic::parse_to_json as parse_basic_mod;
