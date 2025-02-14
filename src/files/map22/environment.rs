use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

/// Season min/max temp with name
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct Season {
    /// minimum temperature
    pub min : i32,
    /// maximum temperature
    pub max : i32,
}

/// Base game weather, FS22
static WEATHER: [Weather; 3] = [
    Weather { // "mapUS",
        spring : Season{ min : 6,   max: 18},
        summer : Season{ min : 13,  max: 34},
        autumn : Season{ min : 5,   max: 25},
        winter : Season{ min : -11, max: 10},
    },
    Weather { // "mapFR",
        spring : Season{ min : 6,   max: 18},
        summer : Season{ min : 13,  max: 34},
        autumn : Season{ min : 5,   max: 25},
        winter : Season{ min : -11, max: 10},
    },
    Weather { // "mapAlpine",
        spring : Season{ min : 5,   max: 18},
        summer : Season{ min : 10,  max: 30},
        autumn : Season{ min : 4,   max: 22},
        winter : Season{ min : -12, max: 8},
    },
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
/// Map weather
pub struct Weather{
    /// spring
    pub spring : Season,
    /// summer
    pub summer : Season,
    /// fall
    pub autumn : Season,
    /// winter
    pub winter : Season,
}

impl XMLReader<Self> for Weather {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        WEATHER[0].clone().read_xml(xml_text).cloned()
    }

    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"environment", 0) => { Ok(1) },
            (b"season", 2) => { self.tag_season(reader, e); Ok(0) },
            (_, 0)        => Err(AbstractFileError::XmlWrongFileType),
            _ => Ok(1),
        }
    }
}

impl Weather {
    /// Get weather from a base game key.
    pub fn from_base<S: AsRef<str>>(name : S) -> Self {
        match name.as_ref() {
            "mapAlpine" => WEATHER[2].clone(),
            "mapFR"     => WEATHER[1].clone(),
            _           => WEATHER[0].clone(),
        }
    }
    /// Do season
    #[inline]
    fn tag_season(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) {
        let mut buf_next = Vec::new();

        let Some(season) = Self::xml_attribute(e, "name") else { return };

        let mut min_temp = i32::MAX;
        let mut max_temp = i32::MIN;

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"variation" => {
                    if let Some(v) = Self::xml_attribute_number(&e, "minTemperature") {
                        min_temp = std::cmp::min(min_temp, v);
                    }
                    if let Some(v) = Self::xml_attribute_number(&e, "maxTemperature") {
                        max_temp = std::cmp::max(max_temp, v);
                    }
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }

        match season.as_str() {
            "spring" => {
                self.spring.max = max_temp;
                self.spring.min = min_temp;
            },
            "summer" => {
                self.summer.max = max_temp;
                self.summer.min = min_temp;
            },
            "autumn" => {
                self.autumn.max = max_temp;
                self.autumn.min = min_temp;
            },
            "winter" => {
                self.winter.max = max_temp;
                self.winter.min = min_temp;
            },
            _ => ()
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
        let actual = Weather::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }

    #[test]
    fn from_file_prod_deep() {
        let filename = "tests/test_mods/MAP_CustomGrowthAndEnvironment.zip";
        let item = "map/xml/environment.xml";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = Weather::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = serde_json::json!({
            "spring": { "min": 6,   "max": 18 },
            "summer": { "min": 13,  "max": 34 },
            "autumn": { "min": 5,   "max": 25 },
            "winter": { "min": -35, "max": 30 }
        });

        let re_read:Weather = serde_json::from_value(expected).unwrap();

        assert_eq!(actual, re_read);
    }

    #[test]
    fn base_game_weather() {
        assert_eq!(Weather::from_base("mapAlpine"), WEATHER[2]);
        assert_eq!(Weather::from_base("mapFR"), WEATHER[1]);
        assert_eq!(Weather::from_base("mapUS"), WEATHER[0]);
        assert_eq!(Weather::from_base("nonsense"), WEATHER[0]);
    }
}