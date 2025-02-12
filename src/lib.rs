#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(missing_docs)]

// pub mod maps;

// pub mod mod_detail;
pub mod savegame;
// pub mod shared;
pub mod parser;

mod errors;
mod files;

/// Known none malware files that fail the general check
pub const NOT_MALWARE: [&str; 16] = [
    "FS25_000_DevTools",
    "FS25_AutoDrive",
    "FS25_Courseplay",
    "FS25_FSG_Companion",
    "FS25_VehicleControlAddon",
    "FS22_001_NoDelete",
    "FS22_AutoDrive",
    "FS22_Courseplay",
    "FS22_FSG_Companion",
    "FS22_VehicleControlAddon",
    "MultiOverlayV3",   // Happylooser
    "MultiOverlayV4",   // Happylooser
    "VehicleInspector", // Happylooser
    "FS19_AutoDrive",
    "FS19_Courseplay",
    "FS19_GlobalCompany",
];

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

/// Options for the parsers
pub struct ParseOptions(Vec<ParseOption>);

impl Default for ParseOptions {
    fn default() -> Self {
        Self(vec![ParseOption::IncludeSaveGame, ParseOption::IncludeDetail, ParseOption::IncludeMap, ParseOption::ImageMod, ParseOption::ImageDetail, ParseOption::ImageMap])
    }
}
impl From<Vec<ParseOption>> for ParseOptions {
    fn from(value: Vec<ParseOption>) -> Self { Self(value) }
}
impl Deref for ParseOptions {
    type Target = Vec<ParseOption>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Options for the parser
pub enum ParseOption {
    /// Include savegame, if found
    IncludeSaveGame,
    /// Include detail scans
    IncludeDetail,
    /// Include Map
    IncludeMap,
    /// Include detail icons
    ImageDetail,
    /// Include mod icon
    ImageMod,
    /// Include map image
    ImageMap,
}

use std::ops::Deref;

pub use savegame::parse as parse_savegame;

pub use parser::parse;
pub use parser::parse_with_options;

pub use parser::parse_detail;
pub use parser::parse_detail_with_options;
