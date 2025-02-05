use std::collections::HashMap;
use crate::errors::AbstractFileError;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

/// Default l10n key
const EN_KEY:&str = "en";
/// Useful actionBinding device
const KB_DEF:&str = "KB_MOUSE_DEFAULT";

/// L10n tags (title, desc)
type ModDescL10n = HashMap<String, String>;
/// Action binding (name, category)
type ModDescAction = HashMap<String, String>;
/// Action keys (name, (key combos))
type ModDescKey = HashMap<String, Vec<String>>;
/// L10N Entries (local) (lang, (key, value))
type ModL10NMap = HashMap<String, HashMap<String, String>>;

/// modDesc.xml struct - XML error handling here, not mod checking
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ModDescXML {
    /// Title, in lang key -> value pairs
    pub title: ModDescL10n,
    /// Description, in lang key -> value pairs
    pub description : ModDescL10n,
    /// action list
    pub actions: ModDescAction,
    /// action bindings name -> key combo(s)
    pub action_binding: ModDescKey,
    /// mod author
    pub author: Option<String>,
    /// script files referenced? (for pc only auto-flag)
    pub script_files: bool,
    /// store items list (xml files)
    pub store_items: Vec<String>,
    /// dependencies (short names)
    pub dependencies: Vec<String>,
    /// desc version
    pub desc_version: u32,
    /// icon filename
    pub icon_filename: Option<String>,
    /// map config file (if map)
    pub map_config_filename: Option<String>,
    /// multiplayer flag
    pub multiplayer: bool,
    /// mod version
    pub version: Option<String>,
    /// prefix for loadable l10n files
    pub l10n_file_prefix: Option<String>,
    /// l10n entries included here
    pub l10n_local: ModL10NMap,
}

impl ModDescXML {
    /// Turn an attribute into a string option
    #[inline]
    fn get_attribute<'a>(e : &'a BytesStart, name : &'a str) -> Option<String> {
        if let Ok(Some(version)) = e.try_get_attribute(name) {
            version.unescape_value().map_or(None, |text| Some(text.to_string()))
        } else {
            None
        }
    }

    /// Turn a [`BytesStart`] name into a string
    #[inline]
    fn key_to_string(e : &BytesStart) -> Option<String> {
        String::from_utf8(e.name().as_ref().to_vec()).ok()
    }

    /// Load the modDesc.xml from an [`super::AbstractFile`]
    pub fn from_abstract_file(mod_file : &mut super::AbstractFile) -> Result<Self, AbstractFileError> {
        let xml_text = mod_file.text("modDesc.xml")?;
        Self::from_string(&xml_text)
    }

    /// Load the modDesc.xml from an already decoded string
    pub fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut mod_desc = Self::default();

        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => return Err(AbstractFileError::XMLParseError),
                Ok(Event::Eof)                    => break,
                Ok(Event::End(_))                 => depth -= 1,
                Ok(Event::Start(e)) => Self::tags_paired(&mut reader, &e, &mut mod_desc, &mut depth)?,
                Ok(Event::Empty(e)) => Self::tags_self_closing(&e, &mut mod_desc, depth),
                _ => ()
            }
        }

        Ok(mod_desc)
    }

    /// Process key bindings
    #[inline]
    fn tag_action_binding(reader: &mut Reader<&[u8]>, mod_desc: &mut Self, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();
        let Some(key) = Self::get_attribute(e, "action") else { return Err(AbstractFileError::XMLParseError) };
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e)) => {
                    if let (Some(d), Some(i)) = (Self::get_attribute(&e, "device"), Self::get_attribute(&e, "input")) {
                        if d == KB_DEF {
                            let key_map = mod_desc.action_binding.entry(key.clone()).or_default();
                            key_map.push(i);
                        }
                    }
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XMLParseError),
                _ => (),
            }
        }
        Ok(())
    }

    /// Process included l10n
    #[inline]
    fn tag_l10n_text(reader: &mut Reader<&[u8]>, mod_desc: &mut Self, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();
        let Some(key) = Self::get_attribute(e, "name") else { return Err(AbstractFileError::XMLParseError) };
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) => {
                    if let (Some(k), Ok(v)) = (Self::key_to_string(&e), reader.read_text(e.name())) {
                        let lang_map = mod_desc.l10n_local.entry(key.clone()).or_default();
                        lang_map.insert(k, v.to_string());
                    }
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XMLParseError),
                _ => (),
            }
        }
        Ok(())
    }

    /// Process description
    #[inline]
    fn tag_description(reader: &mut Reader<&[u8]>, mod_desc: &mut Self, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();
        let mut current_lang = EN_KEY.to_owned();
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) => {
                    current_lang = Self::key_to_string(&e).unwrap_or_else(|| EN_KEY.to_owned());
                },
                Ok(Event::Text(e)) => {
                    if let Ok(v) = e.unescape() {
                        mod_desc.description.insert(current_lang.clone(), v.to_string());
                    }
                }
                Ok(Event::CData(e)) => {
                    if let Ok(v) = String::from_utf8(e.to_vec()) {
                        mod_desc.description.insert(current_lang.clone(), v);
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XMLParseError),
                _ => (),
            }
        }
        Ok(())
    }

    /// Process Title
    #[inline]
    fn tag_title(reader: &mut Reader<&[u8]>, mod_desc: &mut Self, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) => {
                    if let (Some(k), Ok(v)) = (Self::key_to_string(&e), reader.read_text(e.name())) {
                        mod_desc.title.insert(k, v.to_string());
                    }
                },
                Ok(Event::Text(e)) => {
                    if let Ok(v) = e.unescape() {
                        mod_desc.title.insert(EN_KEY.to_owned(), v.to_string());
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XMLParseError),
                _ => (),
            }
        }
        Ok(())
    }
    

    /// Handle paired tags
    #[inline]
    fn tags_paired(reader: &mut Reader<&[u8]>, e: &BytesStart, mod_desc: &mut Self, depth : &mut i32) -> Result<(), AbstractFileError> {
        match e.name().as_ref() {
            b"modDesc" => {
                *depth += 1;
                if let Some(v) = Self::get_attribute(e, "descVersion") {
                    mod_desc.desc_version = v.parse().unwrap_or_default();
                }
            },
            b"author" if *depth == 1 => {
                if let Ok(v) = reader.read_text(e.name()) {
                    mod_desc.author = Some(v.to_string());
                }
            },
            b"version" if *depth == 1 => {
                if let Ok(v) = reader.read_text(e.name()) {
                    mod_desc.version = Some(v.to_string());
                }
            },
            b"iconFilename" if *depth == 1 => {
                if let Ok(v) = reader.read_text(e.name()) {
                    mod_desc.icon_filename = Some(v.to_string());
                }
            },
            b"dependency" if *depth == 2 => {
                if let Ok(v) = reader.read_text(e.name()) {
                    mod_desc.dependencies.push(v.to_string());
                }
            },
            b"map" if *depth == 2 && mod_desc.map_config_filename.is_none() => {
                if let Some(v) = Self::get_attribute(e, "configFilename") {
                    mod_desc.map_config_filename = Some(v);
                }
            },
            b"text"          if *depth == 2 => Self::tag_l10n_text(reader, mod_desc, e)?,
            b"title"         if *depth == 1 => Self::tag_title(reader, mod_desc, e)?,
            b"description"   if *depth == 1 => Self::tag_description(reader, mod_desc, e)?,
            b"actionBinding" if *depth == 2 => Self::tag_action_binding(reader, mod_desc, e)?,
            _ => *depth += 1,
        }
        Ok(())
    }

    /// Handle all self-closing tag
    #[inline]
    fn tags_self_closing(e: &BytesStart, mod_desc: &mut Self, depth : i32) {
        match e.name().as_ref() {
            b"multiplayer" if depth == 1 => {
                if let Some(v) = Self::get_attribute(e, "supported") {
                    mod_desc.multiplayer = v.eq_ignore_ascii_case("true");
                }
            },
            b"l10n" if depth == 1 => {
                if let Some(v) = Self::get_attribute(e, "filenamePrefix") {
                    mod_desc.l10n_file_prefix = Some(v);
                }
            },
            b"sourceFile" if depth == 2 => {
                mod_desc.script_files = true;
            },
            b"storeItem" if depth == 2 => {
                if let Some(v) = Self::get_attribute(e, "xmlFilename") {
                    mod_desc.store_items.push(v);
                }
            },
            b"action" if depth == 2 => {
                if let Some(name) = Self::get_attribute(e, "name") {
                    let category = Self::get_attribute(e, "category").unwrap_or_else(|| String::from("ALL"));
                    mod_desc.actions.insert(name, category);
                }
            },
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;

    #[test]
    fn valid_folder() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/PASS_Good_Simple_Mod");

        let actual = ModDescXML::from_abstract_file(&mut file_handle).expect("process failed");

        // cSpell: disable
        let expected = serde_json::json!({
            "title": {
                "de": "That, in german",
                "en": "Totally valid FS22 Mod"
            },
            "description": {
                "de": "This is wrong",
                "en": "\nDemonstrates how FSModAssist handles a good mod file.\n\n<!-- Some other, commented text (not valid to comment here, still shows) -->\n"
            },
            "actions": {
                "aim_menu": "SYSTEM",
                "aim_hot1": "SYSTEM",
                "aim_togg": "ALL"
            },
            "action_binding": {
                "aim_togg": [
                    "KEY_ralt KEY_KP_0"
                ],
                "aim_hot1": [
                    "KEY_ralt KEY_KP_1"
                ],
                "aim_menu": [
                    "KEY_lshift KEY_slash",
                    "MOUSE_BUTTON_X1"
                ]
            },
            "author": "FSModAssist Test &amp; Bob",
            "script_files": true,
            "store_items": [
                "Dolly.xml"
            ],
            "dependencies": [
                "FS22_RedBarnPack"
            ],
            "desc_version": 69,
            "icon_filename": "modIcon.dds",
            "map_config_filename": "map/xml/map.xml",
            "multiplayer": true,
            "version": "1.0.0.0",
            "l10n_file_prefix": "languages/l10n",
            "l10n_local": {
                "config_5WHardLocking": {
                    "fr": "Verrouillage dur",
                    "de": "Harte Verriegelung"
                },
                "config_5WSemiLocking": {
                    "de": "Teilverriegelung",
                    "en": "Partial locking"
                }
            }
        });

        assert_json_eq!(serde_json::json!(actual), expected);
    }

    #[test]
    fn broken_xml() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/FAILURE_Really_Malformed_ModDesc.zip");

        let actual = ModDescXML::from_abstract_file(&mut file_handle);

        assert_eq!(actual, Err(AbstractFileError::XMLParseError));
    }

    #[test]
    fn broken_zip() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/FAILURE_Bad_ModDesc_CRC.zip");

        let actual = ModDescXML::from_abstract_file(&mut file_handle);

        assert_eq!(actual, Err(AbstractFileError::FileIOError));
    }

    #[test]
    fn missing_file() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/FAILURE_Missing_ModDesc.zip");

        let actual = ModDescXML::from_abstract_file(&mut file_handle);

        assert_eq!(actual, Err(AbstractFileError::FileNotFound));
    }

    #[test]
    fn invalid_but_parseable() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = ModDescXML::from_abstract_file(&mut file_handle).expect("bad file");

        let expected = serde_json::json!({
            "title": {
                "en": "Missing Version"
            },
            "description": {
                "en": "Demonstrates a mod that has does not include a version string"
            },
            "actions": {
                "ENGINESTARTER_SHOW_MENU": "ALL"
            },
            "action_binding": {
                "ENGINESTARTER_SHOW_MENU": [
                    "KEY_lalt KEY_e"
                ]
            },
            "author": "FSModAssist Test",
            "script_files": true,
            "store_items": [],
            "dependencies": [],
            "desc_version": 69,
            "icon_filename": "modIcon.dds",
            "map_config_filename": null,
            "multiplayer": true,
            "version": null,
            "l10n_file_prefix": null,
            "l10n_local": {
                "input_ENGINESTARTER_SHOW_MENU": {
                    "de": "TODO",
                    "cz": "TODO",
                    "pl": "TODO",
                    "fr": "TODO",
                    "it": "TODO",
                    "ru": "TODO",
                    "es": "TODO",
                    "en": "Show Engine Starter Menu",
                    "nl": "TODO",
                    "pt": "TODO"
                },
                "guiTest": {
                    "fr": "TODO",
                    "nl": "TODO",
                    "es": "TODO",
                    "en": "Testing GuI",
                    "de": "TODO",
                    "cz": "TODO",
                    "it": "TODO",
                    "ru": "TODO",
                    "pl": "TODO",
                    "pt": "TODO"
                },
                "engineStartNotification": {
                    "cz": "Startování motoru ...",
                    "nl": "Motor starten ...",
                    "pt": "Partida do motor ...",
                    "it": "Avviamento del motore ...",
                    "pl": "Uruchomienie silnika ...",
                    "en": "Engine starting...",
                    "es": "Arranque del motor ...",
                    "de": "Motor startet ...",
                    "ru": "Запуск двигателя ...",
                    "fr": "Démarrage du moteur ..."
                }
            }
        });
        assert_json_eq!(serde_json::json!(actual), expected);
    }

    #[test]
    fn bad_xml_old_title() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <title>old title</title>
            </modDesc>"#;

        let actual = ModDescXML::from_string(xml).expect("no read");
        let mut expected = HashMap::new();
        expected.insert(String::from("en"), String::from("old title"));

        assert_eq!(actual.title, expected);
    }
}
