#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

pub mod maps;
pub mod mod_basic;
pub mod savegame;
pub mod shared;

pub use savegame::parser as parse_savegame;
pub use savegame::parse_to_json as parse_savegame_json;
pub use savegame::parse_to_json_pretty as parse_savegame_json_pretty;

pub use mod_basic::parser as parse_mod;
pub use mod_basic::parse_to_json as parse_mod_json;
pub use mod_basic::parse_to_json_pretty as parse_mod_json_pretty;
