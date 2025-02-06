// use serde::{Serialize, Deserialize};

/// Possible Detectable Mod Errors
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
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
    /// Mod may contain dangerous files
    InfoDangerousFile,
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

/// Odd, but valid occurrences in XML data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[cfg_attr(test, derive(strum_macros::EnumCount))]
#[serde(rename_all="SCREAMING_SNAKE_CASE")]
pub enum ModDescWarnings {
    /// Invalid tag nested in actionBinding
    ActionBindingInvalidTag(String),
    /// binding missing input or device attribute
    ActionBindingMalformed(),
    /// L10n text entry malformed
    L10nMalformed(),
    /// Invalid language tag (lang, location)
    L10nInvalidLanguage(String, String),
    /// Bare text found in a tag that should be localized
    ShouldBeL10n(String),
}

impl std::fmt::Display for ModDescWarnings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ActionBindingInvalidTag(v) => write!(f, "actionBinding contained invalid key: {v}"),
            Self::ActionBindingMalformed() => write!(f, "actionBinding missing required attributes"),
            Self::L10nMalformed() => write!(f, "malformed l10n text tag"),
            Self::L10nInvalidLanguage(i,l) => write!(f, "unknown l10n language: {i} in {l}"),
            Self::ShouldBeL10n(v) => write!(f, "found un-translated entry in: {v}"),
        }
    }
}


/// File-Level errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[cfg_attr(test, derive(strum_macros::EnumCount))]
#[serde(rename_all="SCREAMING_SNAKE_CASE")]
pub enum AbstractFileError {
    /// File could not be found
    FileNotFound,
    /// File could not be read
    FileIoError,
    /// ZIP archive is not a zip file
    FileNotZip,
    /// ZIP could not be read
    ZipReadError,
    /// Folder could not be found
    FolderError,
    /// Unrecoverable XML parse error
    XmlParseError,
    /// Configured reader could process this xml type
    XmlWrongFileType,
}

impl std::fmt::Display for AbstractFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::FileNotFound => "file not found",
            Self::FileIoError => "file could not be read",
            Self::FileNotZip => "not a zip file",
            Self::ZipReadError => "zip read error",
            Self::FolderError => "folder not accessible",
            Self::XmlParseError => "xml parsing failed",
            Self::XmlWrongFileType => "tried to parse wrong type of xml",
        })
    }
}

impl std::error::Error for AbstractFileError { }
impl From<zip::result::ZipError> for AbstractFileError {
    fn from(value: zip::result::ZipError) -> Self {
        match value {
            zip::result::ZipError::Io(_) => Self::FileIoError,
            zip::result::ZipError::FileNotFound => Self::FileNotFound,
            _ => Self::ZipReadError,
        }
    }
}
impl From<std::io::Error> for AbstractFileError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound,
            _ => Self::FileIoError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::EnumCount;

    #[test]
    fn abstract_file_error_ser() {
        let errors:[AbstractFileError; 7] = [
            AbstractFileError::FileIoError,
            AbstractFileError::FileNotFound,
            AbstractFileError::FileNotZip,
            AbstractFileError::FolderError,
            AbstractFileError::XmlParseError,
            AbstractFileError::ZipReadError,
            AbstractFileError::XmlWrongFileType
        ];

        for e in &errors { assert!(e.to_string().len() > 0); }
        assert_eq!(AbstractFileError::COUNT, errors.len());
        assert_eq!(
            serde_json::to_string(&errors).unwrap(),
            r#"["FILE_IO_ERROR","FILE_NOT_FOUND","FILE_NOT_ZIP","FOLDER_ERROR","XML_PARSE_ERROR","ZIP_READ_ERROR","XML_WRONG_FILE_TYPE"]"#
        );
    }

    #[test]
    fn xml_parse_errors_moddesc() {
        let errors:[ModDescWarnings; 5] = [
            ModDescWarnings::ActionBindingInvalidTag(String::from("xx")),
            ModDescWarnings::ActionBindingMalformed(),
            ModDescWarnings::L10nInvalidLanguage(String::from("xx"), String::from("yy")),
            ModDescWarnings::L10nMalformed(),
            ModDescWarnings::ShouldBeL10n(String::from("xx"))
        ];

        for e in &errors { assert!(e.to_string().len() > 0); }
        assert_eq!(ModDescWarnings::COUNT, errors.len());
        assert_eq!(
            serde_json::to_string(&errors).unwrap(),
            r#"[{"ACTION_BINDING_INVALID_TAG":"xx"},{"ACTION_BINDING_MALFORMED":[]},{"L10N_INVALID_LANGUAGE":["xx","yy"]},{"L10N_MALFORMED":[]},{"SHOULD_BE_L10N":"xx"}]"#
        );
    }
}
