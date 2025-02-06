use crate::errors::AbstractFileError;
use crate::files::{AbstractFile, XMLReader};
use super::{Mod, Mods};


use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Individual farm
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, Default)]
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

impl Career {
    /// Read modDesc from mod file
    fn from_abstract_file(mod_file : &mut AbstractFile) -> Result<Self, AbstractFileError> {
        <Self as XMLReader<Self>>::from_abstract_file(mod_file, "careerSavegame.xml")
    }
}

impl XMLReader<Self> for Career {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut career = Self::default();

        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => return Err(AbstractFileError::XmlParseError),
                Ok(Event::Eof)                    => break,
                Ok(Event::Start(e)) => Self::tags_paired(&mut reader, &e, &mut career, &mut depth)?,
                _ => ()
            }
        }

        Ok(career)
    }

    fn tags_paired(reader: &mut quick_xml::Reader<&[u8]>, e: &quick_xml::events::BytesStart, data: &mut Self, depth : &mut i32) -> Result<(), AbstractFileError> {
        match e.name().as_ref() {
            b"careerSavegame" if *depth == 0 => *depth += 1,
            _        if *depth == 0 => return Err(AbstractFileError::XmlWrongFileType),

            b"mod" if *depth != 0 => {
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
            },
            b"mapTitle" => {
                data.map_title = reader.read_text(e.name()).map(|v|v.to_string()).ok();
            },
            b"savegameName" => {
                data.name = reader.read_text(e.name()).map(|v|v.to_string()).ok();
            },
            b"saveDate" => {
                data.save_date = reader.read_text(e.name()).map(|v|v.to_string()).ok();
            }
            b"playTime" => {
                if let Some(time) = reader.read_text(e.name()).ok().and_then(|v| v.parse::<f64>().ok() ) {
                    let hours = (time / 60_f64).floor();
                    let minutes = ( time % 60_f64).floor();
                    data.play_time = Some(format!("{hours:.0}:{minutes:02.0}"));
                }
            }
            _ => (),
        }
        Ok(())
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
