//! Passable Error flags
use serde::ser::{Serialize, Serializer};

/// Possible Detectable Mod Errors
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum ModError {
	/// File is not the right type for a mod
	FileErrorGarbageFile,
	/// File is probably a copy
	FileErrorLikelyCopy,
	/// File is probably a save game
	FileErrorLikelySaveGame,
	/// File is probably a pack of mods
	FileErrorLikelyZipPack,
	/// Filename is invalid for a mod
	FileErrorNameInvalid,
	/// Filename starts with a digit
	FileErrorNameStartsDigit,
	/// ZIP file could not be read
	FileErrorUnreadableZip,
	/// File is an unsupported archive type
	FileErrorUnsupportedArchive,
	/// Mod may contain pirated material
	InfoLikelyPiracy,
	/// Mod may contain malicious script code
	InfoMaliciousCode,
	/// Mod is unzipped and can't be used in multiplayer
	InfoNoMultiplayerUnzipped,
	/// The modDesc.xml file is damaged
	ModDescDamaged,
	/// The modDesc.xml file is missing
	ModDescMissing,
	/// The mod is missing an icon
	ModDescNoModIcon,
	/// The mod does not have a valid version
	ModDescNoModVersion,
	/// The modDesc.xml file is damaged and could not be parsed
	ModDescParseError,
	/// The modDesc.xml has an old or missing descVersion
	ModDescVersionOldOrMissing,
	/// Some files contain spaces
	PerformanceFileSpaces,
	/// Translated title or description not available
	PerformanceMissingL10N,
	/// File contains DDS files that are too big
	PerformanceOversizeDDS,
	/// File contains GDM files that are too big
	PerformanceOversizeGDM,
	/// File contains I3D.CACHE files that are too big
	PerformanceOversizeI3D,
	/// File contains SHAPES files that are too big
	PerformanceOversizeSHAPES,
	/// File contains XML files that are too big
	PerformanceOversizeXML,
	/// File contains too many extra files
	PerformanceQuantityExtra,
	/// File contains too many GRLE files
	PerformanceQuantityGRLE,
	/// File contains too many PDF files
	PerformanceQuantityPDF,
	/// File contains too many PNG files
	PerformanceQuantityPNG,
	/// File contains too many TXT files
	PerformanceQuantityTXT,
}

/// `ModErrors` the mean a mod is broken (won't work)
pub const BADGE_BROKEN: [&ModError; 10] = [
	&ModError::FileErrorGarbageFile,
	&ModError::FileErrorLikelySaveGame,
	&ModError::FileErrorLikelyZipPack,
	&ModError::FileErrorNameInvalid,
	&ModError::FileErrorNameStartsDigit,
	&ModError::FileErrorUnreadableZip,
	&ModError::FileErrorUnsupportedArchive,
	&ModError::ModDescParseError,
	&ModError::ModDescVersionOldOrMissing,
	&ModError::ModDescMissing,
];

/// `ModErrors` that should be fixed, but probably still work
pub const BADGE_ISSUE: [&ModError; 17] = [
	&ModError::InfoLikelyPiracy,
	&ModError::InfoMaliciousCode,
	&ModError::ModDescNoModIcon,
	&ModError::ModDescNoModVersion,
	&ModError::ModDescDamaged,
	&ModError::PerformanceFileSpaces,
	&ModError::PerformanceMissingL10N,
	&ModError::PerformanceOversizeDDS,
	&ModError::PerformanceOversizeGDM,
	&ModError::PerformanceOversizeI3D,
	&ModError::PerformanceOversizeSHAPES,
	&ModError::PerformanceOversizeXML,
	&ModError::PerformanceQuantityExtra,
	&ModError::PerformanceQuantityGRLE,
	&ModError::PerformanceQuantityPDF,
	&ModError::PerformanceQuantityPNG,
	&ModError::PerformanceQuantityTXT,
];

/// `ModErrors` that denote it's not actually a mod
pub const BADGE_NOT_MOD: [&ModError; 6] = [
	&ModError::FileErrorGarbageFile,
	&ModError::FileErrorLikelySaveGame,
	&ModError::FileErrorLikelyZipPack,
	&ModError::FileErrorUnreadableZip,
	&ModError::FileErrorUnsupportedArchive,
	&ModError::ModDescMissing,
];

impl Serialize for ModError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
			ModError::FileErrorGarbageFile        => serializer.serialize_unit_variant("ModError", 0, "FILE_ERROR_GARBAGE_FILE"),
			ModError::FileErrorLikelyCopy         => serializer.serialize_unit_variant("ModError", 1, "FILE_ERROR_LIKELY_COPY"),
			ModError::FileErrorLikelySaveGame     => serializer.serialize_unit_variant("ModError", 2, "FILE_IS_A_SAVEGAME"),
			ModError::FileErrorLikelyZipPack      => serializer.serialize_unit_variant("ModError", 3, "FILE_ERROR_LIKELY_ZIP_PACK"),
			ModError::FileErrorNameInvalid        => serializer.serialize_unit_variant("ModError", 4, "FILE_ERROR_NAME_INVALID"),
			ModError::FileErrorNameStartsDigit    => serializer.serialize_unit_variant("ModError", 5, "FILE_ERROR_NAME_STARTS_DIGIT"),
			ModError::FileErrorUnreadableZip      => serializer.serialize_unit_variant("ModError", 6, "FILE_ERROR_UNREADABLE_ZIP"),
			ModError::FileErrorUnsupportedArchive => serializer.serialize_unit_variant("ModError", 7, "FILE_ERROR_UNSUPPORTED_ARCHIVE"),
			ModError::InfoLikelyPiracy            => serializer.serialize_unit_variant("ModError", 8, "INFO_MIGHT_BE_PIRACY"),
			ModError::InfoMaliciousCode           => serializer.serialize_unit_variant("ModError", 9, "MALICIOUS_CODE"),
			ModError::InfoNoMultiplayerUnzipped   => serializer.serialize_unit_variant("ModError", 10, "INFO_NO_MULTIPLAYER_UNZIPPED"),
			ModError::ModDescDamaged              => serializer.serialize_unit_variant("ModError", 11, "MOD_ERROR_MODDESC_DAMAGED_RECOVERABLE"),
			ModError::ModDescMissing              => serializer.serialize_unit_variant("ModError", 12, "NOT_MOD_MODDESC_MISSING"),
			ModError::ModDescNoModIcon            => serializer.serialize_unit_variant("ModError", 13, "MOD_ERROR_NO_MOD_ICON"),
			ModError::ModDescNoModVersion         => serializer.serialize_unit_variant("ModError", 14, "MOD_ERROR_NO_MOD_VERSION"),
			ModError::ModDescParseError           => serializer.serialize_unit_variant("ModError", 15, "NOT_MOD_MODDESC_PARSE_ERROR"),
			ModError::ModDescVersionOldOrMissing  => serializer.serialize_unit_variant("ModError", 16, "NOT_MOD_MODDESC_VERSION_OLD_OR_MISSING"),
			ModError::PerformanceFileSpaces       => serializer.serialize_unit_variant("ModError", 17, "PERF_SPACE_IN_FILE"),
			ModError::PerformanceMissingL10N      => serializer.serialize_unit_variant("ModError", 18, "PERF_L10N_NOT_SET"),
			ModError::PerformanceOversizeDDS      => serializer.serialize_unit_variant("ModError", 19, "PERF_DDS_TOO_BIG"),
			ModError::PerformanceOversizeGDM      => serializer.serialize_unit_variant("ModError", 20, "PERF_GDM_TOO_BIG"),
			ModError::PerformanceOversizeI3D      => serializer.serialize_unit_variant("ModError", 21, "PERF_I3D_TOO_BIG"),
			ModError::PerformanceOversizeSHAPES   => serializer.serialize_unit_variant("ModError", 22, "PERF_SHAPES_TOO_BIG"),
			ModError::PerformanceOversizeXML      => serializer.serialize_unit_variant("ModError", 23, "PERF_XML_TOO_BIG"),
			ModError::PerformanceQuantityExtra    => serializer.serialize_unit_variant("ModError", 24, "PERF_HAS_EXTRA"),
			ModError::PerformanceQuantityGRLE     => serializer.serialize_unit_variant("ModError", 25, "PERF_GRLE_TOO_MANY"),
			ModError::PerformanceQuantityPDF      => serializer.serialize_unit_variant("ModError", 26, "PERF_PDF_TOO_MANY"),
			ModError::PerformanceQuantityPNG      => serializer.serialize_unit_variant("ModError", 27, "PERF_PNG_TOO_MANY"),
			ModError::PerformanceQuantityTXT      => serializer.serialize_unit_variant("ModError", 28, "PERF_TXT_TOO_MANY"),
		}
	}
}
