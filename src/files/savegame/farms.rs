use crate::errors::AbstractFileError;
use crate::files::{AbstractFile, XMLReader, XMLReaderDepth};

use quick_xml::events::BytesStart;

/// List of farms
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct Farms(pub Vec<Farm>);


/// Individual farm
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct Farm {
    /// Name of farm
    pub name: String,
    /// Cash on hand for farm
    pub cash: i64,
    /// Loan amount for farm
    pub loan: i64,
    /// Color index for farm (1-16)
    pub color: usize,
}

impl Farm {
    /// New with a name
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self { name : name.as_ref().to_owned(), ..Default::default() }
    }
}

impl XMLReader<Self> for Farms {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut farms = Self::default();
        farms.0.push(Farm::new("--unowned--"));
        farms.read_xml(xml_text).cloned()
    }

    /// Read modDesc from mod file
    fn from_abstract(mod_file : &mut AbstractFile) -> Result<Self, AbstractFileError> {
        Self::from_abstract_file(mod_file, "farms.xml")
    }

    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"farms", 0) => Ok(1),
            (_, 0)        => Err(AbstractFileError::XmlWrongFileType),

            (b"farm", 1) => {
                let mut farm = Farm::default();

                if let Some(v) = Self::xml_attribute(e, "name") {
                    farm.name = v;
                }
                if let Some(v) = Self::xml_attribute_number(e, "color") {
                    farm.color = v;
                }

                #[expect(clippy::cast_possible_truncation)]
                if let Some(v) = Self::xml_attribute_number::<f64>(e, "loan") {
                    farm.loan = v as i64;
                }

                #[expect(clippy::cast_possible_truncation)]
                if let Some(v) = Self::xml_attribute_number::<f64>(e, "money") {
                    farm.cash = v as i64;
                }

                self.0.push(farm);
                Self::slurp(e, reader)
            },
            _ => Ok(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_xml() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = Farms::from_abstract(&mut file_handle);

        assert_eq!(actual, Err(AbstractFileError::FileNotFound));
    }


    #[test]
    fn not_farms() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <barf descVersion="69">
                <description>
                    <en>old title</en>
                </description>
            </barf>"#;

        let actual = Farms::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }
}
