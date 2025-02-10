use std::collections::{HashSet, HashMap};
use crate::errors::{ModDescWarnings, AbstractFileError};
use crate::files::{AbstractFile, XMLReader, XMLReaderDepth, PathType};

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

/// Default l10n key
const EN_KEY:&str = "en";
/// Useful actionBinding device
const KB_DEF:&str = "KB_MOUSE_DEFAULT";
/// Known FS languages
const LANG:[&str; 27] = ["br", "cs", "ct", "cz", "da", "de", "ea", "en", "es", "fc", "fi", "fr", "hu", "id", "it", "jp", "kr", "nl", "no", "pl", "pt", "ro", "ru", "sv", "tr", "uk", "vi"];

/// L10n tags (title, desc)
type ModDescL10n = HashMap<String, String>;
/// Action binding (name, category)
type ModDescAction = HashMap<String, String>;
/// Action keys (name, (key combos))
type ModDescKey = HashMap<String, Vec<String>>;
/// L10N Entries (local) (lang, (key, value))
type ModL10NMap = HashMap<String, HashMap<String, String>>;

/// modDesc.xml struct - XML error handling here, not mod checking
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DescXML {
    /// Warnings from parsing the XML
    pub warnings : HashSet<ModDescWarnings>,
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
    /// brand entries
    pub brands : Vec<DescBrand>,
}

/// modDesc.xml struct - XML error handling here, not mod checking
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DescBrand {
    /// name
    pub name : String,
    /// title
    pub title : String,
    /// icon filename
    pub icon_file : Option<String>,
    /// base game icon
    pub icon_base : Option<String>
}

impl XMLReader<Self> for DescXML {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
    }

    /// Read modDesc from mod file
    fn from_abstract(mod_file : &mut AbstractFile) -> Result<Self, AbstractFileError> {
        Self::from_abstract_file(mod_file, "modDesc.xml")
    }

    /// Handle paired tags
    #[inline]
    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"modDesc", 0) => {
                self.desc_version = Self::xml_attribute(e, "descVersion").and_then(|v| v.parse().ok()).unwrap_or_default();
                Ok(1)
            },
            (_, 0) => Err(AbstractFileError::XmlParseError),
            (b"author", 1) => { self.author = Self::xml_text(e, reader); Ok(0) },
            (b"version", 1) => { self.version = Self::xml_text(e, reader); Ok(0) },
            (b"iconFilename", 1) => { self.icon_filename = Self::xml_text(e, reader); Ok(0) },
            (b"dependency", 2) => {
                if let Some(v) = Self::xml_text(e, reader) {
                    self.dependencies.push(v);
                }
                Ok(0)
            },
            (b"map", 2) if self.map_config_filename.is_none() => {
                self.map_config_filename = Self::xml_attribute(e, "configFilename");
                Ok(0)
            },
            (b"text", 2)          => { self.tag_l10n_text(reader, e)?; Ok(0) },
            (b"title", 1)         => { self.tag_title(reader, e)?; Ok(0) },
            (b"description", 1)   => { self.tag_description(reader, e)?; Ok(0) },
            (b"actionBinding", 2) => { self.tag_action_binding(reader, e)?; Ok(0) },
            _ => Ok(1),
        }
    }

    /// Handle all self-closing tag
    #[inline]
    fn tags_self_closing(&mut self, e: &BytesStart, depth : i32) {
        match (e.name().as_ref(), depth) {
            (b"multiplayer", 1) => {
                if let Some(v) = Self::xml_attribute(e, "supported") {
                    self.multiplayer = v.eq_ignore_ascii_case("true");
                }
            },
            (b"l10n", 1) => {
                if let Some(v) = Self::xml_attribute(e, "filenamePrefix") {
                    self.l10n_file_prefix = Some(v);
                }
            },
            (b"sourceFile", 2) => {
                self.script_files = true;
            },
            (b"storeItem", 2) => {
                if let Some(v) = Self::xml_attribute(e, "xmlFilename") {
                    self.store_items.push(v);
                }
            },
            (b"action", 2) => {
                if let Some(name) = Self::xml_attribute(e, "name") {
                    let category = Self::xml_attribute(e, "category").unwrap_or_else(|| String::from("ALL"));
                    self.actions.insert(name, category);
                }
            },
            (b"brand", 2) => {
                let mut brand = DescBrand {
                    name : Self::xml_attribute(e, "name").unwrap_or_default(),
                    title : Self::xml_attribute(e, "title").unwrap_or_default(),
                    ..Default::default()
                };
                if let Some(image) = Self::xml_attribute(e, "image") {
                    match Self::unwrap_base_path(image) {
                        PathType::Base(v) => {
                            if let Some(path) = v.strip_prefix("store/brands") {
                                brand.icon_base = std::path::Path::new(path).file_stem().map(|v| v.to_string_lossy().to_string());
                            }
                        },
                        PathType::Local(v) => brand.icon_file = Some(v),
                    }
                }
                self.brands.push(brand);
            },
            _ => (),
        }
    }
}

impl DescXML {
    /// Process key bindings
    #[inline]
    fn tag_action_binding(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();
        let Some(key) = Self::xml_attribute(e, "action") else {
            self.warnings.insert(ModDescWarnings::ActionBindingMalformed());
            return Ok(())
        };
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"binding" => {
                    if let (Some(d), Some(i)) = (Self::xml_attribute(&e, "device"), Self::xml_attribute(&e, "input")) {
                        if d == KB_DEF {
                            let key_map = self.action_binding.entry(key.clone()).or_default();
                            key_map.push(i);
                        }
                    } else {
                        self.warnings.insert(ModDescWarnings::ActionBindingMalformed());
                    }
                },
                Ok(Event::Start(e) | Event::Empty(e)) => {
                    self.warnings.insert(ModDescWarnings::ActionBindingInvalidTag(Self::get_key(&e)));
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        }
        Ok(())
    }

    /// Process included l10n
    #[inline]
    fn tag_l10n_text(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();
        let Some(key) = Self::xml_attribute(e, "name") else {
            self.warnings.insert(ModDescWarnings::L10nMalformed());
            return Ok(())
        };
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) if LANG.contains(&Self::get_key(&e).as_str()) => {
                    if let (Some(k), Ok(v)) = (Self::get_key_option(&e), reader.read_text(e.name())) {
                        let lang_map = self.l10n_local.entry(key.clone()).or_default();
                        lang_map.insert(k, v.to_string());
                    }
                },
                Ok(Event::Start(e) | Event::Empty(e)) => {
                    self.warnings.insert(ModDescWarnings::L10nInvalidLanguage(Self::get_key(&e), key.clone()));
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        }
        Ok(())
    }

    /// Process description
    #[inline]
    fn tag_description(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();
        let mut current_lang = EN_KEY.to_owned();
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) => {
                    current_lang = Self::get_key_option(&e).unwrap_or_else(|| EN_KEY.to_owned());
                    if !LANG.contains(&current_lang.as_str()) {
                        self.warnings.insert(ModDescWarnings::L10nInvalidLanguage(current_lang.clone(), String::from("description")));
                    }
                },
                Ok(Event::Text(e)) => {
                    if let Ok(v) = e.unescape() {
                        self.description.insert(current_lang.clone(), v.to_string().replace("\r\n", "\n"));
                    }
                }
                Ok(Event::CData(e)) => {
                    if let Ok(v) = String::from_utf8(e.to_vec()) {
                        self.description.insert(current_lang.clone(), v.replace("\r\n", "\n"));
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        }
        Ok(())
    }

    /// Process Title
    #[inline]
    fn tag_title(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) -> Result<(), AbstractFileError> {
        let mut buf_next = Vec::new();

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) => {
                    if let (Some(k), Ok(v)) = (Self::get_key_option(&e), reader.read_text(e.name())) {
                        if LANG.contains(&k.as_str()) {
                            self.title.insert(k, v.to_string());
                        } else {
                            self.warnings.insert(ModDescWarnings::L10nInvalidLanguage(k.clone(), String::from("title")));
                        }
                    }
                },
                Ok(Event::Text(e)) => {
                    if let Ok(v) = e.unescape() {
                        self.warnings.insert(ModDescWarnings::ShouldBeL10n(String::from("title")));
                        self.title.insert(EN_KEY.to_owned(), v.to_string());
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;

    #[test]
    fn valid_folder() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/PASS_Good_Simple_Mod");
        let actual = DescXML::from_abstract(&mut file_handle).expect("process failed");

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
            },
            "warnings": [],
            "brands" : []
        });

        assert_json_eq!(serde_json::json!(actual), expected);

        let re_read:DescXML = serde_json::from_value(expected).expect("deserialize failed");
        assert_eq!(re_read, actual);
    }

    #[test]
    fn broken_xml() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/FAILURE_Really_Malformed_ModDesc.zip");

        let actual = DescXML::from_abstract(&mut file_handle);

        assert_eq!(actual, Err(AbstractFileError::XmlParseError));
    }

    #[test]
    fn broken_zip() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/FAILURE_Bad_ModDesc_CRC.zip");

        let actual = DescXML::from_abstract(&mut file_handle);

        assert_eq!(actual, Err(AbstractFileError::FileIoError));
    }

    #[test]
    fn missing_file() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/FAILURE_Missing_ModDesc.zip");

        let actual = DescXML::from_abstract(&mut file_handle);

        assert_eq!(actual, Err(AbstractFileError::FileNotFound));
    }

    #[test]
    fn invalid_but_parseable() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/WARNING_No_Version.zip");

        let actual = DescXML::from_abstract(&mut file_handle).expect("bad file");

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
                    "de": "TODO", "cz": "TODO", "pl": "TODO",
                    "fr": "TODO", "it": "TODO", "ru": "TODO", "es": "TODO",
                    "en": "Show Engine Starter Menu", "nl": "TODO", "pt": "TODO"
                },
                "guiTest": {
                    "fr": "TODO", "nl": "TODO", "es": "TODO", "en": "Testing GuI", "de": "TODO",
                    "cz": "TODO", "it": "TODO", "ru": "TODO", "pl": "TODO", "pt": "TODO"
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
            },
            "warnings": [],
            "brands": []
        });
        assert_json_eq!(serde_json::json!(actual), expected);
        let re_read:DescXML = serde_json::from_value(expected).expect("deserialize failed");
        assert_eq!(re_read, actual);
    }

    #[test]
    fn bad_input_binding() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <inputBinding>
                    <actionBinding action="aim_menu">
                        <title>Doesn't belown here</title>
                        <winding device="KB_MOUSE_DEFAULT" input="KEY_wrong" />
                        <binding device="KB_MOUSE_DEFAULT" input="KEY_lshift KEY_slash" />
                        <binding input="KEY_lshift KEY_twenty_seven" />
                        <binding device="KB_MOUSE_DEFAULT" />
                    </actionBinding>
                    <actionBinding baction="aim_menu">
                        <binding device="KB_MOUSE_DEFAULT" input="KEY_lshift KEY_slash" />
                    </actionBinding>
                </inputBinding>
            </modDesc>"#;

        let actual = DescXML::from_string(xml).expect("no read");

        let mut expected = HashMap::new();
        expected.insert(String::from("aim_menu"), vec![String::from("KEY_lshift KEY_slash")]);

        let mut errors = HashSet::new();
        errors.insert(ModDescWarnings::ActionBindingInvalidTag(String::from("title")));
        errors.insert(ModDescWarnings::ActionBindingInvalidTag(String::from("winding")));
        errors.insert(ModDescWarnings::ActionBindingMalformed());

        assert_eq!(actual.action_binding, expected);
        assert_eq!(actual.warnings, errors);

        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <inputBinding>
                    <actionBinding action="aim_menu">
                        <binding device="KB_MOUSE_DEFAULT" input="KEY_lshift KEY_slash" />
                </inputBinding>
            </modDesc>"#;

        let actual = DescXML::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlParseError);
    }

    #[test]
    fn l10n_text_tests() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <l10n>
                    <text name="config_5WSemiLocking">
                        <en>Partial locking</en>
                        <de>Teilverriegelung</de>
                        <ru></ru>
                        <xx>Unknown language</xx>
                        Hi
                    </text>
                    <text lame="ignored_wrong">
                        <en>Partial locking</en>
                    </text>
                </l10n>
            </modDesc>"#;

        let actual = DescXML::from_string(xml).expect("read failed");

        let mut lang_map:ModL10NMap = HashMap::new();
        lang_map.insert(String::from("config_5WSemiLocking"), [("en", "Partial locking"), ("de", "Teilverriegelung"), ("ru", "")].into_iter().map(|(a,b)|(a.to_string(), b.to_string())).collect());

        let mut errors = HashSet::new();
        errors.insert(ModDescWarnings::L10nInvalidLanguage(String::from("xx"), String::from("config_5WSemiLocking")));
        errors.insert(ModDescWarnings::L10nMalformed());

        assert_eq!(actual.l10n_local, lang_map);
        assert_eq!(actual.warnings, errors);

        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <l10n>
                    <text name="config_5WSemiLocking">
                        <en>Partial locking</en>
            </modDesc>"#;

        let actual = DescXML::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlParseError);
    }

    #[test]
    fn bad_xml_old_title() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <title>old title</title>
            </modDesc>"#;

        let actual = DescXML::from_string(xml).expect("no read");
        let mut expected = HashMap::new();
        expected.insert(String::from("en"), String::from("old title"));

        let mut error = HashSet::new();
        error.insert(ModDescWarnings::ShouldBeL10n(String::from("title")));

        assert_eq!(actual.title, expected);
        assert_eq!(actual.warnings, error);
    }

    #[test]
    fn bad_l10n_title() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <title>
                    <en>old title</en>
                    <xx>invalid</xx>
                </title>
            </modDesc>"#;

        let actual = DescXML::from_string(xml).expect("no read");
        let mut expected = HashMap::new();
        expected.insert(String::from("en"), String::from("old title"));

        let mut error = HashSet::new();
        error.insert(ModDescWarnings::L10nInvalidLanguage(String::from("xx"), String::from("title")));

        assert_eq!(actual.title, expected);
        assert_eq!(actual.warnings, error);

        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <title>
                    <en>old title</en>
                    <xx>invalid</xx>
            </modDesc>"#;

        let actual = DescXML::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlParseError);
    }

    #[test]
    fn bad_l10n_desc() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <description>
                    <en>old title</en>
                    <xx>invalid</xx>
                </description>
            </modDesc>"#;

        let actual = DescXML::from_string(xml).expect("no read");
        let mut expected = HashMap::new();
        expected.insert(String::from("en"), String::from("old title"));
        expected.insert(String::from("xx"), String::from("invalid")); // !Special case!

        let mut error = HashSet::new();
        error.insert(ModDescWarnings::L10nInvalidLanguage(String::from("xx"), String::from("description")));

        assert_eq!(actual.description, expected);
        assert_eq!(actual.warnings, error);

        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <modDesc descVersion="69">
                <description>
                    <en>old title</en>
                    <xx>invalid</xx>
            </modDesc>"#;

        let actual = DescXML::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlParseError);
    }

    #[test]
    fn not_mod_desc() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
            <barf descVersion="69">
                <description>
                    <en>old title</en>
                </description>
            </barf>"#;

        let actual = DescXML::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlParseError);
    }

    #[test]
    fn valid_brand() {
        let mut file_handle = super::super::AbstractFile::new("tests/test_mods/DETAIL_Samples.zip");
        let actual = DescXML::from_abstract(&mut file_handle).expect("process failed");

        let expected = serde_json::json!([
            { "name" : "HONEYBEE", "title" : "Honey Bee", "icon_base" : null, "icon_file" : "brand_honeybee.dds" },
            { "name" : "LIZARDLOGISTICS", "title" : "Lizard Logistics", "icon_base" : "brand_lizardLogistics", "icon_file" : null },
        ]);

        assert_eq!(serde_json::json!(actual.brands), expected);

        let re_read:Vec<DescBrand> = serde_json::from_value(expected).expect("deserialize failed");
        assert_eq!(re_read, actual.brands);
    }
}
