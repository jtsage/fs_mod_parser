use std::collections::HashMap;

use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth};
use crate::files::savegame::Mod;
use quick_xml::events::BytesStart;


/// Data structure for savegame mods
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Default, Debug)]
pub struct Mods(pub HashMap<String, Mod>);

impl Mods {
    /// Deep merge another set of mods.
    pub fn deep_merge(&mut self, other: Self) {
        for (key, item) in other.0 {
            let entry = self.0.entry(key).or_default();

            if !item.title.is_empty()   { entry.title.clone_from(&item.title); }
            if !item.version.is_empty() { entry.version.clone_from(&item.version); }
            for farm in item.farms {
                entry.farms.insert(farm);
            }
        }
    }
}

impl XMLReader<Self> for Mods {
    /// Load the vehicles or placables xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
    }

    /// Redefine paired tags processor
    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"vehicles" | b"placeables", 0) => Ok(1),
            (_, 0) => Err(AbstractFileError::XmlWrongFileType),

            (b"vehicle" | b"placeable", 1) => {
                if let Some(name) = Self::xml_attribute(e, "modName") {
                    if let Some(farm) = Self::xml_attribute(e, "farmId").and_then(|v| v.parse::<usize>().ok()) {
                        let entry = self.0.entry(name).or_default();
                        entry.farms.insert(farm);
                    }
                }

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
    fn missing_vehicle_xml() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = Mods::from_abstract_file(&mut file_handle, "vehicles.xml");

        assert_eq!(actual, Err(AbstractFileError::FileNotFound));
    }

    #[test]
    fn missing_placeables_xml() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = Mods::from_abstract_file(&mut file_handle, "placables.xml");

        assert_eq!(actual, Err(AbstractFileError::FileNotFound));
    }


    #[test]
    fn not_either_type() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <barf descVersion="69">
                <description>
                    <en>old title</en>
                </description>
            </barf>"#;

        let actual = Mods::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }
}
