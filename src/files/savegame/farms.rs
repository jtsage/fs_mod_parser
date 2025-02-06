use crate::errors::AbstractFileError;
use crate::files::{AbstractFile, XMLReader, XMLReaderDepth};

use quick_xml::events::BytesStart;

/// List of farms
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, Default)]
pub struct Farms(Vec<Farm>);


/// Individual farm
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, Default)]
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

impl Farms {
    /// Read modDesc from mod file
    fn from_abstract_file(mod_file : &mut AbstractFile) -> Result<Self, AbstractFileError> {
        <Self as XMLReader<Self>>::from_abstract_file(mod_file, "farms.xml")
    }
}

impl XMLReader<Self> for Farms {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut farms = Self::default();
        farms.0.push(Farm::new("--unowned--"));
        farms.read_xml(xml_text).cloned()
    }

    fn tags_paired(e: &BytesStart, depth : i32, data: &mut Self, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"farms", 0) => Ok(1),
            (_, 0)        => Err(AbstractFileError::XmlWrongFileType),

            (b"farm", 1) => {
                let mut farm = Farm::default();

                if let Some(v) = Self::xml_attribute(e, "name") {
                    farm.name = v;
                }
                if let Some(v) = Self::xml_attribute(e, "color") {
                    farm.color = v.parse::<usize>().unwrap_or_default();
                }

                #[expect(clippy::cast_possible_truncation)]
                if let Some(v) = Self::xml_attribute(e, "loan") {
                    farm.loan = v.parse::<f64>().unwrap_or_default() as i64;
                }

                #[expect(clippy::cast_possible_truncation)]
                if let Some(v) = Self::xml_attribute(e, "money") {
                    farm.cash = v.parse::<f64>().unwrap_or_default() as i64;
                }

                let _ = reader.read_to_end(e.to_end().name());
                data.0.push(farm);
                Ok(0)
            },
            _ => Ok(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;

    #[test]
    fn good_file() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/SAVEGAME_Good.zip");

        let actual = Farms::from_abstract_file(&mut file_handle).expect("read fail");

        // cSpell: disable
        let expected = serde_json::json!([
            { "name": "--unowned--", "cash": 0, "loan": 0, "color": 0 },
            { "name": "HENNESSEY ACRES", "cash": 46198, "loan": 230000, "color": 7 },
            { "name": "joinFSG.gg", "cash": 100000, "loan": 0, "color": 1 },
            { "name": "PUBLIC", "cash": 878837, "loan": 0, "color": 8 },
            { "name": "BELLWETHER RANCH", "cash": 110758,"loan": 0,"color": 2 },
            { "name": "THE CROFT", "cash": 42937, "loan": 0, "color": 6 }
        ]);
        // cSpell: enable

        assert_json_eq!(serde_json::json!(actual), expected);
    }

    #[test]
    fn missing_xml() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = Farms::from_abstract_file(&mut file_handle);

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
