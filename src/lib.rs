#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(missing_docs)]

pub mod maps;
pub mod mod_basic;
pub mod mod_detail;
pub mod savegame;
pub mod shared;

mod errors;
mod files;

#[derive(Default)]
#[expect(clippy::struct_excessive_bools)]
/// Parsing options
pub struct ModParserOptions {
    /// Include save game parsing in mod output
    pub include_save_game: bool,
    /// Include detail parsing in mod output
    pub include_mod_detail: bool,
    /// Skip icon processing for detail items
    pub skip_detail_icons: bool,
    /// Skip icon processing for mod
    pub skip_mod_icons: bool,
}

pub use savegame::parser as parse_savegame;

pub use mod_basic::parser as parse_mod;
pub use mod_basic::parser_with_options as parse_mod_with_options;

pub use mod_detail::parser as parse_detail;
pub use mod_detail::parser_with_options as parse_detail_with_options;
