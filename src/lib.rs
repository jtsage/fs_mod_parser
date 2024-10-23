#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]

pub mod maps;
pub mod mod_basic;
pub mod mod_detail;
pub mod savegame;
pub mod shared;

#[derive(Default)]
pub struct ModParserOptions {
	pub include_save_game : bool,
	pub include_mod_detail : bool,
}


pub use savegame::parser as parse_savegame;

pub use mod_basic::parser as parse_mod;
pub use mod_basic::parser_with_options as parse_mod_with_options;

pub use mod_detail::parser as parse_detail;
