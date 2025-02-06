use std::collections::HashMap;

use crate::errors::AbstractFileError;
use crate::files::XMLReader;
use super::Mods;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

impl XMLReader<Self> for Mods {
    /// Load the vehicles or placables xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut mods = Self::default();

        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => return Err(AbstractFileError::XmlParseError),
                Ok(Event::Eof)                    => break,
                Ok(Event::Start(e)) => Self::tags_paired(&mut reader, &e, &mut mods, &mut depth)?,
                _ => ()
            }
        }

        Ok(mods)
    }

    fn tags_paired(reader: &mut quick_xml::Reader<&[u8]>, e: &quick_xml::events::BytesStart, data: &mut Self, depth : &mut i32) -> Result<(), AbstractFileError> {
        match e.name().as_ref() {
            b"vehicles" | b"placeables" if *depth == 0 => *depth += 1,
            _           if *depth == 0 => return Err(AbstractFileError::XmlWrongFileType),

            b"vehicle" | b"placeable" if *depth == 1 => {
                if let Some(name) = Self::xml_attribute(e, "modName") {
                    if let Some(farm) = Self::xml_attribute(e, "farmId").and_then(|v| v.parse::<usize>().ok()) {
                        let entry = data.0.entry(name).or_default();
                        entry.farms.insert(farm);
                    }
                }

                let _ = reader.read_to_end(e.to_end().name());
            },
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
    fn good_vehicles() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/SAVEGAME_Good.zip");

        let actual = ModsVP::from_abstract_file(&mut file_handle, "vehicles.xml").expect("read fail");

        // cSpell: disable
        let expected = serde_json::json!({
            "FS22_36ftLowLoader": { "farms": [ 1 ], "title": "--", "version": "0" },
            "FS22_BackRoadsCounty": { "farms": [ 0 ], "title": "--", "version": "0" },
            "FS22_CaseEcoloTil2500": { "farms": [ 2, 4 ], "title": "--", "version": "0" },
            "FS22_JDBalers": { "farms": [ 4, 5 ], "title": "--", "version": "0" },
            "FS22_JD_HX20": { "farms": [ 1, 5 ], "title": "--", "version": "0" },
            "FS22_KroneBigPack120_80": { "farms": [ 1 ], "title": "--", "version": "0" },
            "FS22_MX_Pack": { "farms": [ 1 ], "title": "--", "version": "0" },
            "FS22_Tanker_Trailer_IMT_525": { "farms": [ 1, 5 ], "title": "--", "version": "0" }
        });
        // cSpell: enable

        assert_json_eq!(serde_json::json!(actual), expected);
    }

    #[test]
    fn good_placables() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/SAVEGAME_Good.zip");

        let actual = ModsVP::from_abstract_file(&mut file_handle, "placeables.xml").expect("read fail");

        // cSpell: disable
        let expected = serde_json::json!({
            "FS22_BackRoadsCounty": { "version": "0", "title": "--", "farms": [0,1,4,5,15]},
            "FS22_ObjectStorage": { "version": "0", "title": "--", "farms": [1,4]},
            "FS22_Machinehall_grainstorage": { "version": "0", "title": "--", "farms": [2]},
            "FS22_openCowPasture": { "version": "0", "title": "--", "farms": [1]},
            "FS22_DutchShedPack": { "version": "0", "title": "--", "farms": [1]},
            "FS22_ExtraLargeSheepBarn": { "version": "0","title": "--", "farms": [4]},
            "FS22_Large_Metal_Pavilion": { "version": "0", "title": "--", "farms": [5]},
            "FS22_Large_Pole_Barn": { "version": "0", "title": "--", "farms": [5]},
            "FS22_hydroGreenhouse": { "version": "0", "title": "--", "farms": [1,5]}
        });
        // cSpell: enable

        assert_json_eq!(serde_json::json!(actual), expected);
    }

    #[test]
    fn missing_vehicle_xml() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = ModsVP::from_abstract_file(&mut file_handle, "vehicles.xml");

        assert_eq!(actual, Err(AbstractFileError::FileNotFound));
    }

    #[test]
    fn missing_placeables_xml() {
        let mut file_handle = crate::files::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = ModsVP::from_abstract_file(&mut file_handle, "placables.xml");

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

        let actual = ModsVP::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }
}
