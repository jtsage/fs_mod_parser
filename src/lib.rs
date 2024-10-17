//! A rust crate for parsing Farming Simulator mod files
//! 
//! Aims to provide a JSON representation of FS mod files
//! optionally including information on contained store
//! items, along with savegame processing, and basic mod
//! pack processing.

pub mod maps;
pub mod mod_basic;
pub mod shared;

pub use mod_basic::parse_to_json as parse_basic_mod;
