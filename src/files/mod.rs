//! Abstract file handler
//!
//! This allows treating zipped mods and unzipped mods
//! the same by the parser
use crate::errors::AbstractFileError;

use glob::glob;
use quick_xml::{events::{BytesStart, Event}, Reader};
use std::{
    fs::{self, File}, io::Read, path::{self, Path, PathBuf}
};

use image::{imageops::FilterType, DynamicImage};
use image_dds::ddsfile;
use std::io::Cursor;
use webp::Encoder;
use base64ct::{Base64, Encoding};

/// modDesc.xml processing
pub mod mod_desc;
/// savegame processing
pub mod savegame;
/// store item processing
pub mod store_item;
/// extra l10n process
pub mod l10n;


/// Abstract file implementation
/// 
/// this allows us to treat folders and zipped mods the same
#[derive(Debug)]
pub enum AbstractFile {
    /// folder on disk
    Folder(PathBuf, FileDefinitions),
    /// zip file
    Zip(zip::ZipArchive<File>, FileDefinitions),
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
                    Self::Null(AbstractFileError::FileIoError), |file| {
                        zip::ZipArchive::new(file).map_or_else(|_| Self::Null(AbstractFileError::ZipReadError), |mut archive| {
                            let file_list = Self::list_zip(&mut archive);
                            Self::Zip(archive, file_list)
                        })
                    })
            } else {
                Self::Null(AbstractFileError::FileNotZip)
            }
        } else {
            path::absolute(path).map_or_else(|_| Self::Null(AbstractFileError::FolderError), |f| {
                Self::Folder(f.clone(), Self::list_folder(&f))
            })
        }
    }

    /// Check if a file exists
    pub fn exists<S: AsRef<str>>(&mut self, filename : S ) -> bool {
        match self {
            Self::Folder(path, _) => path.as_path().join(filename.as_ref()).exists(),
            Self::Zip(archive, _) => archive.by_name(filename.as_ref()).is_ok(),
            Self::Null(_) => false,
        }
    }

    /// Get a file as text
    ///
    /// # Errors
    /// returns [`AbstractFileError::FileIoError`] if file cannot be read
    pub fn text<S: AsRef<str>>(&mut self, filename: S) -> Result<String, AbstractFileError> {
        String::from_utf8(self.bin(filename)?).map_err(|_| AbstractFileError::FileIoError)
    }

    /// Get a file as a binary vector
    /// 
    /// # Errors
    /// returns [`AbstractFileError::FileIoError`] if file cannot be read
    pub fn bin<S: AsRef<str>>(&mut self, filename : S) -> Result<Vec<u8>, AbstractFileError> {
        match self {
            Self::Folder(path, _) => {
                Ok(fs::read(path.as_path().join(filename.as_ref()))?)
            },
            Self::Zip(archive, _) => {
                let mut file = archive.by_name(filename.as_ref())?;
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
    fn list_folder(path: &PathBuf) -> FileDefinitions {
        let mut files: Vec<FileDefinition> = vec![];
        let mut name_index: Vec<String> = vec![];
        let mut size = 0;

        let search = path.clone().join("**/*").to_string_lossy().to_string();
        if let Ok(entries) = glob(&search) {
            for entry in entries.filter_map(Result::ok) {
                let Ok(meta) = std::fs::metadata(&entry) else { continue };
                let Ok(full_path) = path::absolute(&entry) else { continue };

                let relative_path = pathdiff::diff_paths(&full_path, path).map_or_else(|| full_path.clone(), |good_path| good_path);
                let name = Self::posix_path(&relative_path);
                name_index.push(name.clone());
                size += meta.len();

                files.push(FileDefinition{
                    extension: Self::lc_extension(&full_path),
                    path: name,
                    size: meta.len(),
                    is_dir: meta.is_dir(),
                });
            }
        }
        FileDefinitions { files, name_index, size }
    }

    /// List files in a zip
    fn list_zip(archive : &mut zip::ZipArchive<File>) -> FileDefinitions {
        let mut files: Vec<FileDefinition> = vec![];
        let mut size = 0;
        let mut name_index: Vec<String> = vec![];
                
        for i in 0..archive.len() {
            let Ok(file) = archive.by_index(i) else { continue };
            let name = Self::posix_path(&file.mangled_name());

            size += if file.is_dir() { 0 } else { file.size() };
            name_index.push(name.clone());

            files.push(FileDefinition{
                extension: Self::lc_extension(&PathBuf::from(&name)),
                path: name,
                size: if file.is_dir() { 0 } else { file.size() },
                is_dir: file.is_dir(),
            });
        }
        FileDefinitions { files, name_index, size }
    }

    /// Get list of files as [`FileDefinition`]'s
    #[must_use]
    pub fn list(&self) -> Vec<FileDefinition> {
        match self {
            Self::Zip(_, l)| Self::Folder(_, l) => {
                l.files.clone()
            },
            Self::Null(_) => vec![],
        }
    }

    /// Is in file list?
    /// this is not a assurance the file *still* exists for a folder, but
    /// can be used negatively
    pub fn is_in_list<S: Into<String>>(&self, needle: S) -> bool {
        match self {
            Self::Folder(_, l) | Self::Zip(_, l) => l.name_index.contains(&needle.into()),
            Self::Null(_) => false
        }
    }

    /// Get files with the specified prefix
    pub fn name_filter<S: AsRef<str>, T: AsRef<str>>(&self, prefix: S, extension: T) -> Vec<String> {
        let prefix = prefix.as_ref().replace('\\', "/");

        self.list().into_iter()
            .filter(|v| v.extension.eq_ignore_ascii_case(extension.as_ref()) && v.path.starts_with(&prefix))
            .map(|v| v.path)
            .collect()
    }

    /// Get a vec of files with a known extension
    pub fn iter_extensions<S: AsRef<str>>(&self, extension : S) -> Vec<FileDefinition> {
        self.list().into_iter().filter(|v| v.extension.eq_ignore_ascii_case(extension.as_ref())).collect()
    }

    /// Is this a folder?
    #[must_use]
    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Folder(_, _))
    }

    /// Size of file?
    #[must_use]
    pub fn size(&self) -> u64 {
        match self {
            Self::Folder(_, f) | Self::Zip(_, f) => f.size,
            Self::Null(_) => 0,
        }
    }

    /// Load the mod icon, and convert to webp
    ///
    /// Returns the webp as a base64 string suitable for use
    /// with an `<image src="...">` tag.
    ///
    /// Supports DDS BC1-BC7 in one pass, in-memory
    pub fn mod_icon<S: AsRef<str>>(&mut self, needle : S) -> Option<String> {
        let input_file = self.bin(needle).ok()?;
        let input_vector = Cursor::new(input_file);
        let dds = ddsfile::Dds::read(input_vector).ok()?;
        let original_image = image_dds::image_from_dds(&dds, 0).ok()?;
        let unscaled_image = DynamicImage::ImageRgba8(original_image);
        let encoder: Encoder = Encoder::from_image(&unscaled_image).ok()?;
        let webp = encoder.encode(75_f32);
        let b64 = Base64::encode_string(webp.as_ref());
    
        Some(format!("data:image/webp;base64, {b64}"))
    }

    /// Load the map image resize, crop, and convert to webp
    ///
    /// Returns the webp as a base64 string suitable for use
    /// with an `<image src="...">` tag.
    ///
    /// Supports DDS BC1-BC7 in one pass, in-memory
    #[must_use]
    pub fn map_image<S: AsRef<str>>(&mut self, needle : S) -> Option<String> {
        let input_file = self.bin(needle).ok()?;
        let input_vector = Cursor::new(input_file);
        let dds = ddsfile::Dds::read(input_vector).ok()?;
        let original_image = image_dds::image_from_dds(&dds, 0).ok()?;
        let mut unscaled_image = DynamicImage::ImageRgba8(original_image);

        let width = unscaled_image.width();
        let height = unscaled_image.height();

        let cropped_image = unscaled_image
            .crop(width / 4, height / 4, width / 2, height / 2)
            .resize(512, 512, FilterType::Nearest);

        let encoder = Encoder::from_image(&cropped_image).ok()?;
        let webp = encoder.encode(75_f32);
        let b64 = Base64::encode_string(webp.as_ref());

        Some(format!("data:image/webp;base64, {b64}"))
    }


}

/// Used to represent files contained inside an [`AbstractFile`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FileDefinitions {
    /// List of files
    pub files : Vec<FileDefinition>,
    /// index by name
    pub name_index : Vec<String>,
    /// size of archive/folder
    pub size : u64,
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

/// Path type from XML
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathType {
    /// Base game
    Base(String),
    /// Local to the mod
    Local(String)
}

impl PathType {
    /// Get file with dds extension
    #[inline]
    fn to_dds(&self) -> String {
        match self {
            Self::Base(v) | Self::Local(v) => Path::new(v).with_extension("dds").to_string_lossy().to_string()
        }
    }
}

/// XML Reader depth change
pub type XMLReaderDepth = Result<i32, AbstractFileError>;

/// XML Reader
pub trait XMLReader<T> {
    /// Get data from an [`AbstractFile`]
    /// 
    /// # Errors
    /// returns [`AbstractFileError`] if file cannot be read
    fn from_abstract_file<S: AsRef<str>>(mod_file : &mut AbstractFile, needle : S) -> Result<T, AbstractFileError> {
        let xml_text = mod_file.text(needle)?;
        Self::from_string(&xml_text)
    }

    /// Stub for a default file version of [`XMLReader<T>::from_abstract_file`]
    /// 
    /// # Errors
    /// returns [`AbstractFileError`] if file cannot be read
    #[expect(unused_variables)]
    fn from_abstract(mod_file : &mut AbstractFile) -> Result<T, AbstractFileError> { Err(AbstractFileError::FileNotFound) }

    /// Get data from a string
    /// 
    /// # Errors
    /// returns [`AbstractFileError`] if string cannot be parsed
    fn from_string(xml_text: &str) -> Result<T, AbstractFileError>;

    /// read the XML
    /// 
    /// # Errors
    /// returns [`AbstractFileError`] if string cannot be parsed
    fn read_xml(&mut self, xml_text: &str) -> Result<&mut Self, AbstractFileError> {
        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut depth = 0;
        let mut found_xml = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => return Err(AbstractFileError::XmlParseError),
                Ok(Event::Decl(_))                => found_xml = true,
                Ok(Event::Eof)                    => break,
                Ok(Event::End(_))                 => depth -= 1,
                Ok(Event::Start(e)) => {
                    depth += self.tags_paired(&e, depth, &mut reader)?;
                },
                Ok(Event::Empty(e)) => {
                    self.tags_self_closing(&e, depth);
                },
                _ => ()
            }
        }
        if found_xml { Ok(self) } else { Err(AbstractFileError::XmlUndeclared) }
    }

    /// Process paired tags
    /// 
    /// return value is the number of unclosed tags we traversed.
    /// 
    /// # Errors
    /// XML parser errors throw to the caller
    #[expect(unused_variables)]
    #[inline]
    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth { Ok(0) }

    /// Process unpaired tags (no need for reader)
    /// 
    /// no return value
    #[expect(unused_variables)]
    #[inline]
    fn tags_self_closing(&mut self, e: &BytesStart, depth : i32) {}

    /// Get an xml attribute from [`BytesStart`] by name
    #[inline]
    #[must_use]
    fn xml_attribute<'a>(e : &'a BytesStart, name : &'a str) -> Option<String> {
        if let Ok(Some(version)) = e.try_get_attribute(name) {
            version.unescape_value().map_or(None, |text| Some(text.to_string()))
        } else {
            None
        }
    }

    /// Get an xml text node
    #[inline]
    fn xml_text(e: &BytesStart, reader: &mut quick_xml::Reader<&[u8]>) -> Option<String> {
        reader.read_text(e.name()).map(|v|v.to_string()).ok()
    }

    /// Get an xml text node as a number
    #[inline]
    fn xml_number<U>(e: &BytesStart, reader: &mut quick_xml::Reader<&[u8]>) -> Option<U> where 
    U: std::str::FromStr + std::default::Default
    {
        reader.read_text(e.name()).map(|v|v.parse::<U>().unwrap_or_default()).ok()
    }

    /// Get an xml text node as a number
    #[inline]
    #[must_use]
    fn xml_attribute_number<U>(e: &BytesStart, name : &str) -> Option<U> where 
    U: std::str::FromStr + std::default::Default
    {
        Self::xml_attribute(e, name).map(|v| v.parse::<U>().unwrap_or_default())
    }

    /// Slurp and dump children
    /// 
    /// # Errors
    /// can error on XML parse error
    #[inline]
    fn slurp(e: &BytesStart, reader: &mut quick_xml::Reader<&[u8]>) -> Result<i32, AbstractFileError> {
        reader.read_to_end(e.to_end().name()).map(|_| 0).map_err(|_| AbstractFileError::XmlParseError)
    }

    /// Return a tuple of (base path, local path)
    #[inline]
    fn unwrap_base_path<S: AsRef<str>>(path : S) -> PathType {
        path.as_ref().strip_prefix("$data/").map_or_else(|| PathType::Local(path.as_ref().to_owned()), |v| PathType::Base(v.to_owned()))
    }

    /// Turn a [`BytesStart`] name into a string
    #[inline]
    #[must_use]
    fn get_key_option(e : &BytesStart) -> Option<String> {
        String::from_utf8(e.name().as_ref().to_vec()).ok()
    }

    /// Turn a [`BytesStart`] name into a string (forced)
    #[inline]
    #[must_use]
    fn get_key(e : &BytesStart) -> String {
        String::from_utf8(e.name().as_ref().to_vec()).unwrap_or_default()
    }
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
    fn not_zip() {
        let mut file_handle = AbstractFile::new("tests/test_mods/FAILURE_Garbage_File.txt");

        assert!(matches!(file_handle, AbstractFile::Null(AbstractFileError::FileNotZip)));

        assert_eq!(file_handle.exists("modDesc.xml"), false);
        assert_eq!(file_handle.bin("modDesc.xml"), Err(AbstractFileError::FileNotZip));
        assert_eq!(file_handle.list(), vec![]);
        assert_eq!(file_handle.is_in_list("bob.txt"), false);
        assert_eq!(file_handle.iter_extensions("xml"), vec![]);

        let file_handle = AbstractFile::new("tests/test_mods/FAILURE_Broken_Zip_File.zip");
        assert!(matches!(file_handle, AbstractFile::Null(AbstractFileError::ZipReadError)));
    }

    #[test]
    fn valid_zip() {
        let mut file_handle = AbstractFile::new("tests/test_mods/PASS_Good_Simple_Mod.zip");

        assert!(matches!(file_handle, AbstractFile::Zip(_, _)));
        assert_eq!(file_handle.is_dir(), false);

        let expected_files = vec![
            FileDefinition { extension: String::from("xml"), path: String::from("modDesc.xml"), size: 2852, is_dir: false },
            FileDefinition { extension: String::from("dds"), path: String::from("modIcon.dds"), size: 32896, is_dir: false }
        ];

        assert_eq!(file_handle.list(), expected_files);
        assert!(file_handle.is_in_list("modDesc.xml"));
        assert_eq!(file_handle.is_in_list("bob.txt"), false);
        assert_eq!(file_handle.iter_extensions("xml"), expected_files[..1]);
        assert_eq!(file_handle.exists("modIcon.dds"), true);
        let mod_desc_text = file_handle.text("modDesc.xml").expect("file open failed");
        assert_eq!(mod_desc_text.len(), 2852);
        assert_eq!(file_handle.bin("modDesc.bad"), Err(AbstractFileError::FileNotFound));
    }

    #[test]
    fn valid_folder() {
        let mut file_handle = AbstractFile::new("tests/test_mods/PASS_Good_Simple_Mod");

        assert!(matches!(file_handle, AbstractFile::Folder(_, _)));
        assert_eq!(file_handle.is_dir(), true);

        let required_files = vec![String::from("modDesc.xml"), String::from("modIcon.dds")];

        for file in file_handle.list() {
            assert!(required_files.contains(&file.path), "file-not-found {}", file.path);
        }
        assert_eq!(file_handle.exists("modIcon.dds"), true);
        assert_eq!(file_handle.is_in_list("bob.txt"), false);
        assert_eq!(file_handle.iter_extensions("xml").len(), 1);
        let mod_desc_text = file_handle.text("modDesc.xml").expect("file open failed");
        assert!(mod_desc_text.len() > 1000);

        assert_eq!(file_handle.bin("modDesc.bad"), Err(AbstractFileError::FileNotFound));
    }
}
