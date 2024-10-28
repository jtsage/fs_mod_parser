#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::redundant_type_annotations)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_to_string)]
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]

pub mod maps;
pub mod mod_basic;
pub mod mod_detail;
pub mod savegame;
pub mod shared;

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
