use std::collections::HashMap;

use crate::errors::AbstractFileError;
use crate::files::{AbstractFile, XMLReader};

/// L10n files
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all="camelCase")]
pub struct L10n(pub HashMap<String, HashMap<String, String>>);

/// single l10n file
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all="camelCase")]
pub struct L10nFile(pub HashMap<String, String>);

impl L10n {
    /// Load l10n files from an [`AbstractFile`]
    pub fn from_abstract_folder<S: AsRef<str>>(mod_file : &mut AbstractFile, folder : S) -> Self {
        let mut value_map = Self::default();

        let mut folder = folder.as_ref().to_owned();
        folder.push('_');

        let files = mod_file.name_filter(&folder, "xml");

        for file in files {
            let Ok(data) = L10nFile::from_abstract_file(mod_file, &file) else { continue };

            let Some(lang_tag) = file.strip_prefix(&folder) else { continue };
            let Some(lang_tag) = lang_tag.strip_suffix(".xml") else { continue };
            let lang_tag = lang_tag.to_owned();

            for (k, v) in data.0 {
                let lang_entry = value_map.0.entry(lang_tag.clone()).or_default();
                lang_entry.insert(k, v);
            }
        }

        value_map
    }
}

impl XMLReader<Self> for L10nFile {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
    }

    fn tags_self_closing(&mut self, e: &quick_xml::events::BytesStart, _depth : i32) {
        match e.name().as_ref() {
            b"text" => {
                let Some(name) = Self::xml_attribute(e, "name") else { return };
                let Some(value) = Self::xml_attribute(e, "text") else { return };
                self.0.insert(name, value);
            },
            b"e" => {
                let Some(name) = Self::xml_attribute(e, "k") else { return };
                let Some(value) = Self::xml_attribute(e, "v") else { return };
                self.0.insert(name, value);
            },
            _ => (),
        }
    }
}

// MARK: TESTING
#[cfg(test)]
mod tests {
    use super::*;

    fn xml_test(str: &str) -> String {
        format!("<?xml version=\"1.0\" ?>\n{str}")
    }

    #[test]
    fn standard() {
        let xml = xml_test(r#"
        <l10n><texts>
            <text name="name_01" text="value_01" />
            <text name="name_02" text="value_02" />
        </texts></l10n>
        "#);

        let actual = L10nFile::from_string(&xml).unwrap();

        assert_eq!(actual.0.get(&String::from("name_01")), Some(&String::from("value_01")));
        assert_eq!(actual.0.get(&String::from("name_02")), Some(&String::from("value_02")));
        assert_eq!(actual.0.len(), 2);
    }

    #[test]
    fn concise() {
        let xml = xml_test(r#"
        <l10n><elements>
            <e k="name_01" v="value_01" />
            <e k="name_02" v="value_02" />
        </elements></l10n>
        "#);

        let actual = L10nFile::from_string(&xml).unwrap();

        assert_eq!(actual.0.get(&String::from("name_01")), Some(&String::from("value_01")));
        assert_eq!(actual.0.get(&String::from("name_02")), Some(&String::from("value_02")));
        assert_eq!(actual.0.len(), 2);
    }
}
