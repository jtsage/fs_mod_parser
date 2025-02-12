//! Main mod parser
use std::collections::HashSet;
use std::path::{self, Path};
use md5::{Md5, Digest};
use base64ct::{Base64UrlUnpadded, Encoding};
use std::io::{Read, Seek, SeekFrom};

use crate::files::{AbstractFile, XMLReader};
use crate::files::mod_desc::DescXML;

use crate::ParseOptions;
use crate::errors::{AbstractFileError, ModError};

/// one megabyte
const MB: u64 = 0x0010_0000;
/// max size allowed for I3D Cache files, 10MB
const SIZE_CACHE: u64 = 10 * MB;
/// max size allowed for DDS files, 12MB
const SIZE_DDS: u64 = 12 * MB;
/// max size allowed for GDM files, 18 MB
const SIZE_GDM: u64 = 18 * MB;
/// max size allowed for SHAPES files, 256MB
const SIZE_SHAPES: u64 = 256 * MB;
/// max size allowed for XML files, 256KB / 0.25MB
const SIZE_XML: u64 = MB / 4;

/// max allowed GRLE files
const MAX_GRLE: u32 = 10;
/// max allowed PDF files
const MAX_PDF: u32 = 1;
/// max allowed PNG files
const MAX_PNG: u32 = 128;
/// max allowed TXT files
const MAX_TXT: u32 = 2;


/// file record
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// Full path to file
    pub file: FileInfo,
    /// Mod ident from full path and filename (MD5)
    pub ident: String,
    //// Is a folder record?
    // pub badge_array: ModBadges,
    /// Mod not usable flag
    pub can_not_use: bool,
    /// Current collection for mod (not set)
    pub current_collection: String,
    // /// Detail icons processed flag
    // pub detail_icon_loaded: bool,
    // /// File details
    // pub file_detail: ModFile,
    /// Errors or issues found
    pub issues: HashSet<ModError>,
    // /// storeItems found (if processed)
    // pub include_detail: Option<ModDetail>,
    // /// save game record (if processed)
    // pub include_save_game: Option<bool>, //FIXME:
    // /// L10N title and description
    // pub l10n: ModDescL10N,
    // /// MD5 Sum (not yet implemented)
    // pub md5_sum: Option<String>,
    /// modDesc.xml fields
    pub mod_desc: DescXML,
    // /// Mod UUID from full path and filename (MD5)
    // pub uuid: String,
}

impl Record {
    /// make a new record
    fn new<P: AsRef<Path>>(filename: P) -> Self {
        let full_path = path::absolute(&filename).unwrap_or_else(|_| filename.as_ref().to_path_buf());

        let mut ident = Md5::new();
        ident.update(full_path.to_string_lossy().as_ref());
        let ident = Base64UrlUnpadded::encode_string(&ident.finalize());

        Self {
            ident,
            file : FileInfo::new(full_path),
            ..Default::default()
        }
    }

    /// Check file name
    fn check_name(&mut self) {
        if !self.file.is_folder {
            if let Some(ext) = Path::new(&self.file.full_path).extension() {
                match ext.to_ascii_lowercase().as_encoded_bytes() {
                    b"zip" => (),
                    b"rar" | b"7z" => {
                        self.issues.insert(ModError::FileErrorUnsupportedArchive);
                        self.issues.insert(ModError::FileErrorNameInvalid);
                        self.can_not_use = true;
                    },
                    _ => {
                        self.issues.insert(ModError::FileErrorGarbageFile);
                        self.issues.insert(ModError::FileErrorNameInvalid);
                        self.can_not_use = true;
                    }
                }
            }
        }

        if self.file.short_name.to_ascii_lowercase().contains("unzip") {
            self.issues.insert(ModError::FileErrorLikelyZipPack);
        }

        if self.file.short_name.starts_with(|c: char| c.is_ascii_digit()) {
            self.issues.insert(ModError::FileErrorNameStartsDigit);
            self.issues.insert(ModError::FileErrorNameInvalid);
            self.can_not_use = true;
        }

        if !self.file.short_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' ) {
            let copy_name: Vec<&str> = self.file.short_name
                .split_inclusive(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .map(str::trim)
                .collect();

            if copy_name[0].chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
            {
                self.issues.insert(ModError::FileErrorLikelyCopy);
                self.file.copy_name = Some(copy_name[0].to_owned());
            }
            
            self.issues.insert(ModError::FileErrorNameInvalid);
            self.can_not_use = true;
        }
    }
}


/// File related metadata for a mod
#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    /// Age hash - MD5 of {shortname}-{size}-{date}
    pub age_hash: String,
    /// suggested name if this appears to be a copy of a mod
    pub copy_name: Option<String>,
    /// list of extra files in mod
    pub extra_files: Vec<String>,
    /// mod file date
    pub file_date: String,
    /// mod size (packed zip or folder contents)
    pub file_size: u64,
    /// full path to file
    pub full_path: String,
    /// list of I3D files
    pub i3d_files: Vec<String>,
    /// list of DDS files
    #[serde(rename = "imageDDS")]
    pub image_dds: Vec<String>,
    /// list of non DDS images
    #[serde(rename = "imageNonDDS")]
    pub image_non_dds: Vec<String>,
    /// folder flag (is this a folder?)
    pub is_folder: bool,
    /// save game flag (is this a save game?)
    pub is_save_game: bool,
    /// mod pack flag (is this a pack of mods?)
    pub is_mod_pack: bool,
    /// list of PNG textures (false positives possible)
    pub png_texture: Vec<String>,
    /// short name of mod (the bit before the .zip extension, or the folder name)
    pub short_name: String,
    /// list of files with spaces in them
    pub space_files: Vec<String>,
    /// list of oversized files
    pub too_big_files: Vec<String>,
    // /// list of zip files
    // pub zip_files: Vec<ZipPackFile>,
}

/// Default hash value (for invalid caching items)
pub const DEFAULT_HASH:&str = "ERR-HASH-NOT-COMPUTED--";

impl FileInfo {
    /// Create a new fileinfo (from abs path)
    fn new<P: AsRef<Path>>(filename : P) -> Self {
        Self {
            full_path  : filename.as_ref().to_string_lossy().to_string(),
            short_name : filename.as_ref().file_stem().map(|v| v.to_string_lossy().to_string() ).unwrap_or_default(),
            is_folder  : filename.as_ref().is_dir(),
            age_hash   : Self::hash_from_file(filename).unwrap_or_else(|_| DEFAULT_HASH.to_owned()),
            ..Default::default()
        }
    }

    /// compute hash from filename
    fn hash_from_file<P: AsRef<Path>>(filename : P) -> Result<String, std::io::Error>{
        let Some(name) = filename.as_ref().file_name() else { return Ok(DEFAULT_HASH.to_owned()) };
        let stats = std::fs::metadata(&filename)?;
        let mut file = std::fs::File::open(&filename)?;
        let mut buf = vec![0; 2048];
    
        file.seek(SeekFrom::End(-2048))?;
        file.read_exact(&mut buf)?;
    
        let mut hasher = Md5::new();
        hasher.update(name.as_encoded_bytes());
        hasher.update(stats.len().to_be_bytes());
        hasher.update(buf);
        let hash = hasher.finalize();
        Ok(Base64UrlUnpadded::encode_string(&hash))
    }
}

/// Parse a mod file
pub fn parse<P: AsRef<Path>>(filename: P) -> Record {
    parse_with_options(filename, ParseOptions::default())
}

/// Parse a mod file with defined options
pub fn parse_with_options<P: AsRef<Path>>(filename: P, options : ParseOptions) -> Record {
    let record = mod_parser(filename, options);
    //record.do_stuff();
    record
}

/// private version of the parser, for post-processing in one place.
fn mod_parser<P: AsRef<Path>>(filename: P, _options : ParseOptions) -> Record {
    let mut record = Record::new(filename);

    record.check_name();

    let mut file = AbstractFile::new(&record.file.full_path);

    match file {
        AbstractFile::Null(AbstractFileError::ZipReadError) => {
            record.issues.insert(ModError::FileErrorUnreadableZip);
            record.can_not_use = true;
            return record
        },
        AbstractFile::Null(_) => {
            record.issues.insert(ModError::FileErrorUnreadable);
            record.can_not_use = true;
            return record
        },
        _ => (),
    }

    match DescXML::from_abstract(&mut file) {
        Ok(mod_desc) => {
            record.mod_desc = mod_desc;
        },
        Err(AbstractFileError::XmlParseError) => {
            record.issues.insert(ModError::ModDescParseError);
            record.can_not_use = true;
            return record
        },
        Err(_) => {
            record.issues.insert(ModError::ModDescMissing);
            record.can_not_use = true;
            return record
        }
    };

    record
}