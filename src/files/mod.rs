//! Abstract file handler
//!
//! This allows treating zipped mods and unzipped mods
//! the same by the parser
use crate::errors::AbstractFileError;

use glob::glob;
use std::{
    fs::{self, File},
    io::Read,
    path::{self, Path, PathBuf},
};

/// modDesc.xml processing
pub mod mod_desc;


/// Abstract file implementation
/// 
/// this allows us to treat folders and zipped mods the same
#[derive(Debug)]
pub enum AbstractFile {
    /// folder on disk
    Folder((PathBuf, Vec<FileDefinition>)),
    /// zip file
    Zip((zip::ZipArchive<File>, Vec<FileDefinition>)),
    /// file or folder failure
    Null(AbstractFileError)
}

impl AbstractFile {
    /// New [`AbstractFile`] from path
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        if ! path.exists() {
            Self::Null(AbstractFileError::FileNotFound)
        } else if ! path.is_dir() {
            if path.extension().map(|ext| ext.eq_ignore_ascii_case("zip")) == Some(true) {
                std::fs::File::open(path).map_or_else(|_|
                    Self::Null(AbstractFileError::ZipReadError), |file| {
                        zip::ZipArchive::new(file).map_or_else(|_| Self::Null(AbstractFileError::ZipReadError), |mut archive| {
                            let file_list = Self::list_zip(&mut archive);
                            Self::Zip((
                                archive,
                                file_list
                            ))
                        })
                    })
            } else {
                Self::Null(AbstractFileError::FileNotZip)
            }
        } else {
            path::absolute(path).map_or_else(|_| Self::Null(AbstractFileError::FolderError), |f| {
                Self::Folder((
                    f.clone(),
                    Self::list_folder(&f)
                ))
            })
        }
    }

    /// Check if a file exists
    pub fn exists(&mut self, filename : String ) -> bool {
        match self {
            Self::Folder((path, _)) => path.as_path().join(filename).exists(),
            Self::Zip((archive, _)) => archive.by_name(filename.as_str()).is_ok(),
            Self::Null(_) => false,
        }
    }

    /// Get a file as text
    pub fn text(&mut self, filename: &str) -> Result<String, AbstractFileError> {
        String::from_utf8(self.bin(filename)?).map_err(|_| AbstractFileError::FileIOError)
    }

    /// Get a file as a binary vector
    pub fn bin(&mut self, filename : &str) -> Result<Vec<u8>, AbstractFileError> {
        match self {
            Self::Folder((path, _)) => {
                Ok(fs::read(path.as_path().join(filename))?)
            },
            Self::Zip((archive, _)) => {
                let mut file = archive.by_name(filename)?;
                let mut buf = vec![];
                file.read_to_end(&mut buf)?;
                Ok(buf.clone())
            },
            Self::Null(v) => Err(*v),
        }
    }

    /// Force an extension to lowercase
    fn lc_extension(filename: &Path) -> String {
        filename.extension().map_or_else(String::new, |v| v.to_string_lossy().to_ascii_lowercase())
    }

    /// Force a path to posix-like (zip required, windows works fine as long as it's relative)
    fn posix_path(filename: &Path) -> String {
        filename.to_string_lossy().into_owned().replace('\\', "/")
    }

    /// List files in a folder
    fn list_folder(path: &PathBuf) -> Vec<FileDefinition> {
        let mut files: Vec<FileDefinition> = vec![];

        let search = path.clone().join("**/*").to_string_lossy().to_string();
        if let Ok(entries) = glob(&search) {
            for entry in entries.filter_map(Result::ok) {
                let Ok(meta) = std::fs::metadata(&entry) else { continue };
                let Ok(full_path) = path::absolute(&entry) else { continue };

                let relative_path = pathdiff::diff_paths(&full_path, path).map_or_else(|| full_path.clone(), |good_path| good_path);

                files.push(FileDefinition{
                    extension: Self::lc_extension(&full_path),
                    path: Self::posix_path(&relative_path),
                    size: meta.len(),
                    is_dir: meta.is_dir(),
                });
            }
        }
        files
    }

    /// List files in a zip
    fn list_zip(archive : &mut zip::ZipArchive<File>) -> Vec<FileDefinition> {
        let mut files: Vec<FileDefinition> = vec![];
                
        for i in 0..archive.len() {
            let Ok(file) = archive.by_index(i) else { continue };
            let name = Self::posix_path(&file.mangled_name());

            files.push(FileDefinition{
                extension: Self::lc_extension(&PathBuf::from(&name)),
                path: name,
                size: if file.is_dir() { 0 } else { file.size() },
                is_dir: file.is_dir(),
            });
        }
        files
    }

    /// Get list of files as [`FileDefinition`]'s
    pub fn list(&self) -> Vec<FileDefinition> {
        match self {
            Self::Zip((_, l)) | Self::Folder((_, l)) => l.clone(),
            Self::Null(_) => vec![],
        }
    }

    /// Is this a folder?
    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Folder(_))
    }

    /// Get moddesc file
    pub fn get_mod_desc(&mut self) -> Result<mod_desc::ModDescXML, AbstractFileError> {
        mod_desc::ModDescXML::from_abstract_file(self)
    }
}


/// Used to represent a file contained inside an [`AbstractFile`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDefinition {
    /// File extension, forced to lowercase
    pub extension: String,
    /// *Relative* file path, including extension
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Folder flag (is this a folder?)
    pub is_dir: bool,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_path() {
        let file_handle = AbstractFile::new("./foo/bar/foo");

        assert!(matches!(file_handle, AbstractFile::Null(AbstractFileError::FileNotFound)));
    }

    #[test]
    fn valid_zip() {
        let mut file_handle = AbstractFile::new("tests/test_mods/PASS_Good_Simple_Mod.zip");

        assert!(matches!(file_handle, AbstractFile::Zip(_)));
        assert_eq!(file_handle.is_dir(), false);

        let expected_files = vec![
            FileDefinition { extension: String::from("xml"), path: String::from("modDesc.xml"), size: 2852, is_dir: false },
            FileDefinition { extension: String::from("dds"), path: String::from("modIcon.dds"), size: 32896, is_dir: false }
        ];

        assert_eq!(file_handle.list(), expected_files);
        assert_eq!(file_handle.exists(String::from("modIcon.dds")), true);
        let mod_desc_text = file_handle.text("modDesc.xml").expect("file open failed");
        assert_eq!(mod_desc_text.len(), 2852);
    }

    #[test]
    fn valid_folder() {
        let mut file_handle = AbstractFile::new("tests/test_mods/PASS_Good_Simple_Mod");

        assert!(matches!(file_handle, AbstractFile::Folder(_)));
        assert_eq!(file_handle.is_dir(), true);

        let expected_files = vec![
            FileDefinition { extension: String::from("xml"), path: String::from("modDesc.xml"), size: 3508, is_dir: false },
            FileDefinition { extension: String::from("dds"), path: String::from("modIcon.dds"), size: 32896, is_dir: false }
        ];

        assert_eq!(file_handle.list(), expected_files);
        assert_eq!(file_handle.exists(String::from("modIcon.dds")), true);
        let mod_desc_text = file_handle.text("modDesc.xml").expect("file open failed");
        assert_eq!(mod_desc_text.len(), 3508);

        assert!(file_handle.get_mod_desc().is_ok());
    }
}
