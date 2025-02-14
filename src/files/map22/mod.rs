use std::fmt;
use quick_xml::events::BytesStart;
use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth};

/// Environment (weather)
pub mod environment;

/// Map Definition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct Config {
    /// overview image data
    pub image_data : Option<String>,
    /// overview image filename
    pub image_file : Option<String>,

    /// Fruit types
    /// <fruitTypes filename="$data/maps/mapUS/maps_fruitTypes.xml" />
    pub fruit_types_file : Option<String>,
    /// key, if base game info 
    pub fruit_types_base : Option<String>,
    
    /// Environment
    /// <environment filename="$data/maps/mapUS/environment.xml" />
    pub environment_file : Option<String>,
    /// key, if base game info
    pub environment_base : Option<String>,
    
    /// Growth file
    /// <growth filename="$data/maps/mapUS/maps_growth.xml" />
    pub growth_file: Option<String>,
    /// key, if base game info
    pub growth_base: Option<String>,
}

impl XMLReader<Self> for Config {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::new().read_xml(xml_text).cloned()
    }

    fn tags_paired(&mut self, e: &BytesStart, depth : i32, _reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"map", 0) => {
                if let Some(image_file) = Self::xml_attribute(e, "imageFilename") {
                    let image_file = Self::unwrap_base_path(image_file);
                    self.image_file = Some(image_file.to_dds());
                }
                Ok(1)
            },
            (_, 0)        => Err(AbstractFileError::XmlWrongFileType),
            _ => Ok(1),
        }
    }

    fn tags_self_closing(&mut self, e: &BytesStart, _depth : i32) {
        match e.name().as_ref() {
            b"fruitTypes" => {
                let (base, local) = Self::resolve_data_file(Self::xml_attribute(e, "filename"));
                self.fruit_types_base = base;
                self.fruit_types_file = local;
            },
            b"environment" => {
                let (base, local) = Self::resolve_data_file(Self::xml_attribute(e, "filename"));
                self.environment_base = base;
                self.environment_file = local;
            },
            b"growth" => {
                let (base, local) = Self::resolve_data_file(Self::xml_attribute(e, "filename"));
                self.growth_base = base;
                self.growth_file = local;
            },
            _ => (),
        }
    }
}

impl Config {
    /// Make new map record
    fn new() -> Self {
        Self {
            fruit_types_base: Some(String::from("mapUS")),
            environment_base: Some(String::from("mapUS")),
            growth_base: Some(String::from("mapUS")),
            ..Default::default()
        }
    }

    /// Resolve data resource to local file or base game data
    fn resolve_data_file(filename: Option<String>) -> (Option<String>, Option<String>) {
        filename.map_or_else(|| (Some(String::from("mapUS")), None), |filename| match Self::unwrap_base_path(filename) {
                super::PathType::Base(v) => match v {
                    v if v.contains("mapFR") => (Some(String::from("mapFR")), None),
                    v if v.contains("mapAlpine") => (Some(String::from("mapAlpine")), None),
                    _ => (Some(String::from("mapUS")), None),
                },
                super::PathType::Local(v) => (None, Some(v)),
            })
    }
}

/// Period mask
/// 
/// Used specifically to store month-based period data
/// notes:
/// - the bit order is intentionally reversed so that `0b0000_0000_0001` is period #12
/// - periods are 1-based, as in the FS xml
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]
#[serde(into="Vec<u8>", from="Vec<u8>")]
pub struct PeriodMask(u16);

impl PeriodMask {
    /// Set a bit (1-based, chainable)
    #[inline]
    pub fn set(&mut self, n:u8) -> &mut Self {
        self.0 |= 1 << (12-n);
        self
    }
    /// get a bit (1-based)
    #[must_use]
    #[inline]
    pub fn get(&self, n:u8) -> bool {
        self.0 & (1 << (n-1)) != 0
    }
}

impl fmt::Debug for PeriodMask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PeriodMask({:#014b})", self.0)
    }
}
impl From<PeriodMask> for Vec<bool> {
    fn from(value: PeriodMask) -> Self {
        (1..=12).rev().map(|y| value.get(y)).collect()
    }
}
impl From<PeriodMask> for Vec<u8> {
    fn from(value: PeriodMask) -> Self {
        #[expect(clippy::cast_possible_truncation)]
        (1..=12).rev()
            .map(|y| value.get(y))
            .enumerate()
            .filter(|&v| v.1)
            .map(|v| (v.0 + 1) as u8)
            .collect()
    }
}
impl From<Vec<u8>> for PeriodMask {
    fn from(value: Vec<u8>) -> Self {
        let mut period = Self::default();
        for i in value { period.set(i); }
        period
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn xml_test(str: &str) -> String {
        format!("<?xml version=\"1.0\" ?>\n{str}")
    }

    #[test]
    fn wrong_type() {
        let xml = xml_test(r#"
            <garbage>
                <storeData><image>$data/path/to/data/file.png</image></storeData>
            </garbage>"#);
        let actual = Config::from_string(xml.as_ref());
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }

    #[test]
    fn base_game_all() {
        let xml = xml_test(r#"
            <map imageFilename="maps/overview.png">
                <environment filename="$data/maps/mapAlpine/environment.xml" />
                <growth filename="$data/maps/mapFR/maps_fruitTypes.xml" />
                <fruitTypes filename="$data/maps/mapUS/maps_fruitTypes.xml" />
            </map>
        "#);

        let actual = Config::from_string(&xml).unwrap();
        
        assert_eq!(actual.image_file, Some(String::from("maps/overview.dds")));
        assert_eq!(actual.environment_base, Some(String::from("mapAlpine")));
        assert_eq!(actual.growth_base, Some(String::from("mapFR")));
        assert_eq!(actual.fruit_types_base, Some(String::from("mapUS")));
        assert_eq!(actual.environment_file, None);
        assert_eq!(actual.growth_file, None);
        assert_eq!(actual.fruit_types_file, None);
    }

    #[test]
    fn custom_all() {
        let xml = xml_test(r#"
            <map imageFilename="maps/overview.png">
                <environment filename="map/environment.xml" />
                <growth filename="map/maps_fruitTypes.xml" />
                <fruitTypes filename="map/maps_fruitTypes.xml" />
            </map>
        "#);

        let actual = Config::from_string(&xml).unwrap();
        
        assert_eq!(actual.image_file, Some(String::from("maps/overview.dds")));
        assert_eq!(actual.environment_file, Some(String::from("map/environment.xml")));
        assert_eq!(actual.growth_file, Some(String::from("map/maps_fruitTypes.xml")));
        assert_eq!(actual.fruit_types_file, Some(String::from("map/maps_fruitTypes.xml")));
        assert_eq!(actual.environment_base, None);
        assert_eq!(actual.growth_base, None);
        assert_eq!(actual.fruit_types_base, None);
    }


    #[test]
    fn bit_mask() {
        let mut bits = PeriodMask::default();

        bits.set(4).set(12);

        assert_eq!(bits.0, 0b0001_0000_0001);
        
        let bools:Vec<bool> = bits.into();
        let months:Vec<u8> = bits.into();

        assert_eq!(bools, vec![false, false, false, true, false, false, false, false, false, false, false, true]);
        assert_eq!(months, vec![4,12]);

        assert_eq!(&format!("{bits:?}"), "PeriodMask(0b000100000001)");
        assert_eq!(serde_json::to_string(&bits).unwrap(), String::from("[4,12]"));
        let re_read:PeriodMask = serde_json::from_str("[4,12]").unwrap();
        assert_eq!(re_read, bits);
    }
}