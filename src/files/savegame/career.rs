use crate::errors::AbstractFileError;
use crate::files::{AbstractFile, XMLReader, XMLReaderDepth};
use super::{Mod, items::Mods};

/// Save career
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, Default)]
#[serde(rename_all="camelCase")]
pub struct Career {
    /// Map mod name (shortname)
    pub map_mod: Option<String>,
    /// Map title
    pub map_title: Option<String>,
    /// Number of mods loaded
    pub mod_count: usize,
    /// List of mods
    pub mods: Mods,
    /// Name of the save
    pub name: Option<String>,
    /// Playtime in hours:minutes, hours is unbound
    pub play_time: Option<String>,
    /// Save date
    pub save_date: Option<String>,
}

impl XMLReader<Self> for Career {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
    }

    /// Read modDesc from mod file
    fn from_abstract(mod_file : &mut AbstractFile) -> Result<Self, AbstractFileError> {
        Self::from_abstract_file(mod_file, "careerSavegame.xml")
    }

    fn tags_self_closing(e: &quick_xml::events::BytesStart, depth : i32, data: &mut Self) {
        if e.name().as_ref() == b"mod" && depth == 1 {
            if let Some(name) = Self::xml_attribute(e, "modName") {
                let  mut entry = Mod::default();
                if let Some(title) = Self::xml_attribute(e, "title") {
                    entry.title = title;
                }
                if let Some(version) = Self::xml_attribute(e, "version") {
                    entry.version = version;
                }
                data.mod_count += 1;
                data.mods.0.insert(name, entry);
            }
        }
    }

    fn tags_paired(e: &quick_xml::events::BytesStart, depth : i32, data: &mut Self, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"careerSavegame", 0) => Ok(1),
            (_, 0) => Err(AbstractFileError::XmlWrongFileType),

            (b"mapTitle", 2) => {
                data.map_title = reader.read_text(e.name()).map(|v|v.to_string()).ok();
                Ok(0)
            },
            (b"savegameName", 2) => {
                data.name = reader.read_text(e.name()).map(|v|v.to_string()).ok();
                Ok(0)
            },
            (b"saveDate", 2) => {
                data.save_date = reader.read_text(e.name()).map(|v|v.to_string()).ok();
                Ok(0)
            }
            (b"playTime", 2) => {
                if let Some(time) = reader.read_text(e.name()).ok().and_then(|v| v.parse::<f64>().ok() ) {
                    let hours = (time / 60_f64).floor();
                    let minutes = ( time % 60_f64).floor();
                    data.play_time = Some(format!("{hours:.0}:{minutes:02.0}"));
                }
                Ok(0)
            },
            (b"mapId", 2) => {
                if let Ok(id) = reader.read_text(e.name()) {
                    data.map_mod = id.split('.').next().map(std::string::ToString::to_string);
                }
                Ok(0)
            }
            _ => Ok(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_include;

    #[test]
    fn good_file() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/SAVEGAME_Good.zip");

        let actual = Career::from_abstract(&mut file_handle).expect("read fail");

        // cSpell: disable
        let expected = serde_json::json!({
            "mapMod": "FS22_BackRoadsCounty",
            "mapTitle": "Back Roads County",
            "modCount": 38,
            "mods": {
                "FS22_2150_Series": { "farms": [], "title": "Case IH 2150 Early Riser Planters Series", "version": "1.0.0.0" },
                "FS22_25DU_Trailers": { "farms": [], "title": "Lizard 25DU Trailer", "version": "1.0.0.0" }
            },
            "name": "BRC", 
            "playTime": "306:40",
            "saveDate": "2022-10-14"
        });
        // cSpell: enable

        assert_json_include!(actual : serde_json::json!(actual), expected : expected);
    }

    #[test]
    fn missing_xml() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = Career::from_abstract(&mut file_handle);

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

        let actual = Career::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }
}
