//! Abstract file handler
//! 
//! This allows treating zipped mods and unzipped mods
//! the same by the parsers
use std::{fs::{self, File}, io::Read, path::{self, Path, PathBuf}};
use glob::glob;
use crate::shared::errors::ModError;

/// Used to represent a file contained inside an [`AbstractFileHandle`]
#[derive(Debug)]
pub struct FileDefinition {
    /// File extension, forced to lowercase
    pub extension : String,
    /// File name, including extension
    pub name : String,
    /// File size in bytes
    pub size : u64,
    /// Folder flag (is this a folder?)
    pub is_folder : bool,
}

/// Use a folder or zip file interchangeably
pub trait AbstractFileHandle {
    /// Check if a file exists in the zip/folder
    fn exists(&mut self, needle : &str) -> bool;

    /// Is this a folder (or a zip file)
    fn is_folder(&self) -> bool;

    /// List contained files
    fn list(&mut self) -> Vec<FileDefinition>;

    /// Open a contained file as text
    /// 
    /// # Errors
    /// 
    /// returns as error when file not found or unreadable
    fn as_text(&mut self, needle : &str) -> Result<String, std::io::Error>;

    /// Open a contained file as binary
    /// 
    /// # Errors
    /// 
    /// returns as error when file not found or unreadable
    fn as_bin(&mut self, needle : &str) -> Result<Vec<u8>, std::io::Error>;
}


/// Open a folder as an [`AbstractFileHandle`]
pub struct AbstractFolder {
    /// [`PathBuf`] to folder
    path : PathBuf
}

impl AbstractFolder {
    /// Create a new [`AbstractFileHandle`] record from a folder [`std::path::Path`]
    ///
    /// # Errors
    /// 
    /// Can possibly return [`ModError::FileErrorUnreadableZip`] - should be added direct
    /// to mod record issues.
    pub fn new<P: AsRef<Path>>(file_path :P) -> Result<AbstractFolder, ModError> {
        let input_path = file_path.as_ref();

        if input_path.exists() {
            Ok(AbstractFolder { path : input_path.to_path_buf() })
        } else {
            Err(ModError::FileErrorUnreadableZip)
        }
    }
}
impl AbstractFileHandle for AbstractFolder {
    fn as_text(&mut self, needle : &str) -> Result<String, std::io::Error> {
        let search_path = Path::new(&self.path).join(needle);
        fs::read_to_string(search_path)
    }
    fn as_bin(&mut self, needle : &str) -> Result<Vec<u8>, std::io::Error> {
        let search_path = Path::new(&self.path).join(needle);
        fs::read(search_path)
    }
    fn is_folder(&self) -> bool { true }
    fn list(&mut self) -> Vec<FileDefinition> {
        let mut names: Vec<FileDefinition> = vec![];

        for entry in glob(format!("{}/**/*", self.path.to_string_lossy()).as_str()).unwrap().filter_map(Result::ok) {
            let file_metadata = std::fs::metadata(&entry).unwrap();
            let full_path = path::absolute(entry).unwrap();
            let relative_path = match pathdiff::diff_paths(&full_path, &self.path) {
                Some(good_path) => good_path.to_str().unwrap().to_owned(),
                None => full_path.to_str().unwrap().to_owned(),
            };

            let extension = match full_path.extension() {
                Some(ext) => ext.to_str().unwrap().to_owned().to_ascii_lowercase(),
                None => String::new(),
            };

            names.push(FileDefinition{
                extension,
                name : relative_path.replace('\\', "/"),
                size : file_metadata.len(),
                is_folder : file_metadata.is_dir(),
            });
        }

        names
    }
    fn exists(&mut self, needle : &str) -> bool {
        let search_path = Path::new(&self.path).join(needle);

        search_path.exists()
    }
}

/// Open a zip file as an [`AbstractFileHandle`]
pub struct AbstractZipFile {
    /// archive file (opened)
    archive : zip::ZipArchive<File>
}
impl AbstractZipFile {
    /// Create a new [`AbstractFileHandle`] record from a zip file [`std::path::Path`]
    /// 
    /// # Errors
    /// 
    /// Can possibly return [`ModError::FileErrorUnreadableZip`] - should be added direct
    /// to mod record issues.
    pub fn new<P: AsRef<Path>>(file_path :P) -> Result<AbstractZipFile, ModError> {
        let path = file_path.as_ref();
        match std::fs::File::open(path) {
            Ok(file) => {
                match zip::ZipArchive::new(file) {
                    Ok(archive) => {
                        Ok(AbstractZipFile {
                            archive
                        })
                    },
                    Err(..) => {
                        Err(ModError::FileErrorUnreadableZip)
                    },
                }
            },
            Err(..) => {
                Err(ModError::FileErrorUnreadableZip)
            },
        }
    }
}
impl AbstractFileHandle for AbstractZipFile {
    fn as_bin(&mut self, needle : &str) -> Result<Vec<u8>, std::io::Error> {
        let mut file = self.archive.by_name(needle)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        Ok(buf.clone())
    }

    fn as_text(&mut self, needle : &str) -> Result<String, std::io::Error> {
        let mut file = self.archive.by_name(needle)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
    fn is_folder(&self) -> bool { false }
    fn list(&mut self) -> Vec<FileDefinition> {
        let mut names: Vec<FileDefinition> = vec![];
        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i).unwrap();
            let name = file.mangled_name().to_string_lossy().into_owned().replace('\\', "/");

            let extension = match Path::new(&name).extension() {
                Some(ext) => ext.to_str().unwrap().to_owned().to_ascii_lowercase(),
                None => String::new(),
            };

            names.push(FileDefinition{
                extension,
                name,
                size      : if file.is_dir() {0} else { file.size() },
                is_folder : file.is_dir()
            });
        }
        names
    }
    fn exists(&mut self, needle : &str) -> bool {
        match self.archive.by_name(needle) {
            Ok(..) => true,
            Err(..) => false,
        }
    }
}


/// Open nothing as an [`AbstractFileHandle`]
pub struct AbstractNull {}

impl AbstractNull {
    /// Create a new [`AbstractFileHandle`] record from a null
    /// 
    /// Only used for testing purposes
    ///
    /// # Errors
    /// 
    /// Never returns an error, but all [`AbstractFileHandle`] implementations 
    /// either fail (reads) or return empty (list)
    pub fn new() -> Result<AbstractNull, ModError> {
        Ok(AbstractNull{})
    }
}
#[allow(unused_variables)]
impl AbstractFileHandle for AbstractNull {
    fn as_text(&mut self, needle : &str) -> Result<String, std::io::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not implemented"))
    }
    fn as_bin(&mut self, needle : &str) -> Result<Vec<u8>, std::io::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not implemented"))
    }
    fn is_folder(&self) -> bool { false }
    fn list(&mut self) -> Vec<FileDefinition> { vec![] }
    fn exists(&mut self, needle : &str) -> bool { false }
}