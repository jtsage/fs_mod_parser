use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth};

use quick_xml::events::BytesStart;


/// Map Definition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct Map22 {
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

impl XMLReader<Self> for Map22 {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
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


impl Map22 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::AbstractFile;

    #[test]
    fn working() {
        let mut file = AbstractFile::new("./tests/test_mods/MAP_NoCustoms");

        let map = Map22::from_abstract_file(&mut file, "maps/map.xml");

        println!("{map:?}")
    }

}