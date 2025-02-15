use std::{collections::HashMap, ops::Deref, sync::LazyLock};
use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

/// Fruit type
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all="camelCase")]
pub struct FruitType {
    /// Last valid harvest state
    pub max_harvest: usize,
    /// First valid harvest state
    pub min_harvest: usize,
}

impl FruitType {
    /// Create crop record where all values match
    #[must_use]
    pub const fn singleton(v : usize) -> Self {
        Self { max_harvest : v, min_harvest : v }
    }
}

/// Base game fruit types - FS22
static FRUIT_TYPES: LazyLock<HashMap<String, FruitType>> = LazyLock::new(|| {
    HashMap::from([
        (String::from("wheat"),         FruitType::singleton(8)),
        (String::from("barley"),        FruitType::singleton(7)),
        (String::from("canola"),        FruitType::singleton(9)),
        (String::from("oat"),           FruitType::singleton(5)),
        (String::from("maize"),         FruitType::singleton(7)),
        (String::from("sunflower"),     FruitType::singleton(8)),
        (String::from("soybean"),       FruitType::singleton(7)),
        (String::from("potato"),        FruitType::singleton(6)),
        (String::from("sugarbeet"),     FruitType::singleton(8)),
        (String::from("sugarcane"),     FruitType::singleton(8)),
        (String::from("cotton"),        FruitType::singleton(9)),
        (String::from("sorghum"),       FruitType::singleton(5)),
        (String::from("grape"),         FruitType { max_harvest : 11, min_harvest : 10 }),
        (String::from("olive"),         FruitType { max_harvest : 10, min_harvest : 9 }),
        (String::from("poplar"),        FruitType { max_harvest : 14, min_harvest : 14 }),
        (String::from("grass"),         FruitType { max_harvest : 4, min_harvest : 3 }),
        (String::from("oilseedradish"), FruitType { max_harvest : 2, min_harvest : 2 })
    ])
});

/// Fruit types, from base or file
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct FruitTypes(HashMap<String, FruitType>);

impl Deref for FruitTypes {
    type Target = HashMap<String, FruitType>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl XMLReader<Self> for FruitTypes {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
    }

    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"map", 0) => { Ok(1) },
            (b"fruitType", 2) => { self.tag_fruit(reader, e); Ok(0) },
            (_, 0)        => Err(AbstractFileError::XmlWrongFileType),
            _ => Ok(1),
        }
    }
}

impl FruitTypes {
    /// Get weather from a base game key.
    pub fn from_base() -> Self {
        Self(FRUIT_TYPES.clone())
    }
    /// Do season
    #[inline]
    fn tag_fruit(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) {
        let mut buf_next = Vec::new();

        let Some(fruit) = Self::xml_attribute(e, "name") else { return };

        let mut min_harvest = 1_usize;
        let mut max_harvest = 1_usize;

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e) | Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"harvest" => {
                            if let Some(v) = Self::xml_attribute_number(&e, "minHarvestingGrowthState") {
                                min_harvest = v;
                            }
                            if let Some(v) = Self::xml_attribute_number(&e, "maxHarvestingGrowthState") {
                                max_harvest = v;
                            }
                        },
                        b"preparing" => {
                            if let Some(v) = Self::xml_attribute_number(&e, "minGrowthState") {
                                min_harvest = v;
                            }
                            if let Some(v) = Self::xml_attribute_number(&e, "maxGrowthState") {
                                max_harvest = v;
                            }
                        },
                        _ => (),
                    }
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }

        if fruit.to_ascii_lowercase() != *"meadow" {
            self.0.insert(fruit.to_ascii_lowercase(), FruitType { max_harvest, min_harvest });
        }

    }
}


// MARK: TESTING
#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::AbstractFile;
    use pretty_assertions::assert_eq;

    #[test]
    fn wrong_type() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?><barf></barf>"#;
        let actual = FruitTypes::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }

    #[test]
    fn from_file() {
        let filename = "tests/test_mods/MAP_AddedCrops.zip";
        let item = "maps/xml/maps_fruitTypes.xml";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = FruitTypes::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = serde_json::json!({
            "barley": { "maxHarvest": 7, "minHarvest": 7 },
            "grape": { "maxHarvest": 11, "minHarvest": 10 },
            "potato": { "maxHarvest": 6, "minHarvest": 6 },
            "sorghum": { "maxHarvest": 5, "minHarvest": 5 },
            "sunflower": { "maxHarvest": 8, "minHarvest": 8 },
            "oat": { "maxHarvest": 5, "minHarvest": 5 },
            "sugarbeet": { "maxHarvest": 8, "minHarvest": 8 },
            "clover": { "maxHarvest": 7, "minHarvest": 5 },
            "alfalfa": { "maxHarvest": 7, "minHarvest": 5 },
            "sugarcane": { "maxHarvest": 8, "minHarvest": 8 },
            "oilseedradish": { "maxHarvest": 2, "minHarvest": 2 },
            "silage_corn": { "maxHarvest": 7, "minHarvest": 7 },
            "cotton": { "maxHarvest": 9, "minHarvest": 9 },
            "soybean": { "maxHarvest": 7, "minHarvest": 7 },
            "grass": { "maxHarvest": 4, "minHarvest": 3 },
            "wheat": { "maxHarvest": 8, "minHarvest": 8 },
            "olive": { "maxHarvest": 10, "minHarvest": 9 },
            "canola": { "maxHarvest": 9, "minHarvest": 9 },
            "maize": { "maxHarvest": 7, "minHarvest": 7 },
            "poplar": { "maxHarvest": 14, "minHarvest": 14 }
        });


        let re_read:FruitTypes = serde_json::from_value(expected).unwrap();

        assert_eq!(actual, re_read);
    }

    #[test]
    fn base_game_fruits() {
        assert_eq!(*FruitTypes::from_base(), FRUIT_TYPES.clone());
    }
}
