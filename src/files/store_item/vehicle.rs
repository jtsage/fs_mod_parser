use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth, PathType};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

// MARK: Vehicle
/// Vehicle storeItem record
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    /// fill configs
    pub fills : Vec<FillConfig>,
    /// sprays
    pub sprays : Vec<SprayType>,
    /// feature flags
    pub flags: Flags,
    /// path to base game icon
    pub icon_base: Option<String>,  // TODO: icon_base
    /// path to local icon
    pub icon_file: Option<String>, //TODO: icon_file
    /// icon data
    pub icon_data: Option<String>, //TODO: icon_data
    // motor information
    // pub motor: ModDetailVehicleEngine, // TODO: motor
    /// File is a sub of a different item
    pub parent_item: Option<String>,
    /// sorting information
    pub sorting: Sorting,
}

// MARK: Sorting
/// Vehicle sorting data
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Sorting {
    /// brand KEY
    pub brand: Option<String>,
    /// category
    pub category: Option<String>,
    /// list of combos (local or basegame)
    pub combos: Vec<Combo>,
    /// name of vehicle
    pub name: Option<String>,
    /// type name
    pub type_name: Option<String>,
    /// type description
    pub type_description: Option<String>,
    /// year of vehicle (non-standard)
    pub year: Option<u32>,
    /// vehicle functions
    pub functions: Vec<String>,
    /// this vehicle can use tools that want to connect to these joints
    pub joint_accepts: Vec<String>,
    /// this vehicle needs to connect to these type of joints
    pub joint_requires: Vec<String>,
    /// vehicle price
    pub price: Option<u32>,
    /// vehicle weight
    pub weight: u32,
    /// max speed
    pub max_speed: Option<u32>,
    /// needed power
    pub needed_power: Option<u32>,
    /// power
    pub power: Option<u32>,
    /// speed limit
    pub speed_limit: Option<f32>,
    /// working width
    pub working_width: Option<f32>,
}


/// Vehicle combo data
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "lowercase")]
pub enum Combo {
    /// Base game category
    Category(String),
    /// Filename - local
    Local(String),
    /// Filename - base game
    Base(String)
}


// MARK: Flags
///Capability
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[serde(from="bool", into="bool")]
pub enum Capability {
    /// Has option
    Yes,
    /// Does not have option
    #[default]
    No,
}

impl From<Capability> for bool {
    fn from(value: Capability) -> Self { matches!(value, Capability::Yes) }
}
impl From<bool> for Capability {
    fn from(value: bool) -> Self { if value { Self::Yes } else { Self::No } }
}

/// Vehicle flags
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    /// has beacon lights
    pub beacons: Capability,
    /// has paint options
    pub color: Capability,
    /// can be entered by player
    pub enterable: Capability,
    /// has real lights
    pub lights: Capability,
    /// is motorized
    pub motorized: Capability,
    /// has wheel options
    pub wheels: Capability,
}

// MARK: FillConfig
/// Fill unit in a config
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct FillUnit {
    /// fill categories
    categories: Vec<String>,
    /// fill types
    types : Vec<String>,
    /// fill capacity
    capacity : u32,
}

/// Fill unit config
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct FillConfig(Vec<FillUnit>);


/// Vehicle spray variant
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SprayType {
    /// fill types supported
    pub fills: Option<Vec<String>>,
    /// working width
    pub width: Option<f32>,
}

// MARK: Motors

// MARK: XMLReader
impl XMLReader<Self> for Vehicle {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut vehicle = Self::default().read_xml(xml_text).cloned()?;

        vehicle.sorting.joint_accepts.sort();
        vehicle.sorting.joint_accepts.dedup();
        vehicle.sorting.joint_requires.sort();
        vehicle.sorting.joint_requires.dedup();
        Ok(vehicle)
    }

    #[inline]
    fn tags_paired(e: &BytesStart, depth : i32, data: &mut Self, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"vehicle", 0) => {
                data.sorting.type_name = Self::xml_attribute(e, "type");
                Ok(1)
            },
            (_, 0) => Err(AbstractFileError::XmlWrongFileType),

            (b"image", 2) => {
                if let Some(filename) = Self::xml_text(e, reader) {
                    match Self::unwrap_base_path(filename) {
                        PathType::Base(v) => data.icon_base = Some(v),
                        PathType::Local(v) => data.icon_file = Some(v),
                    }
                }
                Ok(0)
            },
            (b"parentFile", 1) => { data.parent_item = Self::xml_text(e, reader); Ok(0) },

            // MARK: ~sorting
            (b"brand", 2) => { data.sorting.brand = Self::xml_text(e, reader); Ok(0) },
            (b"category", 2) => { data.sorting.category = Self::xml_text(e, reader); Ok(0) },
            (b"name", 2) => { data.sorting.name = Self::xml_text(e, reader); Ok(0) },
            (b"typeDesc", 2) => { data.sorting.type_description = Self::xml_text(e, reader); Ok(0) },
            (b"year", 2) => {
                data.sorting.year = Self::xml_number(e, reader);
                Ok(0)
            },
            (b"function", 3) => { 
                if let Some(v) = Self::xml_text(e, reader) { data.sorting.functions.push(v) }
                Ok(0)
            },
            (b"price", 2) => { data.sorting.price = Self::xml_number(e, reader); Ok(0) },
            (b"speedLimit", 2) => { data.sorting.price = Self::xml_attribute(e, "value").map(|v| v.parse().unwrap_or_default()); Ok(0) },
            (b"specs", 2) => Self::tag_specs(reader, data, e),

            // MARK: ~flags

            (b"motorized", 1) => {
                data.flags.motorized = Capability::Yes; Ok(1)
            },
            (b"enterable", 1) => {
                data.flags.enterable = Capability::Yes; Self::slurp(e, reader)
            },
            (b"realLights", 2) => {
                data.flags.lights = Capability::Yes; Self::slurp(e, reader)
            },
            (b"beaconLights", 2) => {
                data.flags.beacons = Capability::Yes; Self::slurp(e, reader)
            },
            (b"baseMaterialConfigurations" | b"baseColorConfigurations" | b"designMaterialConfigurations" | b"designMaterial2Configurations" | b"designMaterial3Configurations", 1) => {
                data.flags.color = Capability::Yes; Self::slurp(e, reader)
            },
            (v, 1) if v.starts_with(b"designColorConfigurations") => {
                data.flags.color = Capability::Yes; Self::slurp(e, reader)
            },
            (b"wheels", 1) => Self::tag_wheels(reader, data, e),
            (b"attacherJoint", _) => {
                if let Some(joint) = Self::xml_attribute(e, "jointType") {
                    data.sorting.joint_accepts.push(joint);
                }
                Self::slurp(e, reader)
            }
            (b"inputAttacherJoint", _) => {
                if let Some(joint) = Self::xml_attribute(e, "jointType") {
                    data.sorting.joint_requires.push(joint);
                }
                Self::slurp(e, reader)
            }

            // MARK: ~fill/spray

            (b"sprayType", 3) => Self::tag_spray_type(reader, data, e),
            (b"fillUnitConfiguration", 3) => Self::tag_fill_config(reader, data, e),
            // MARK: ~motors
            // (b"motorConfigurations", 2) => { Self::tag_wheels(reader, data, e)?; Ok(0) },


            _ => Ok(1),
        }
    }

    fn tags_self_closing(e: &BytesStart, depth : i32, data: &mut Self) {
        match (e.name().as_ref(), depth) {
            (b"component", 3) => {
                if let Some(mass) = Self::xml_attribute(e, "mass").map(|v| v.parse::<u32>().unwrap_or_default()) {
                    data.sorting.weight += mass;
                }
            },
            (b"inputAttacherJoint", _) => {
                if let Some(joint) = Self::xml_attribute(e, "jointType") {
                    data.sorting.joint_requires.push(joint);
                }
            }
            _ => (),
        }
    }
}

impl Vehicle {
    // MARK: _wheels
    /// Do wheels
    #[inline]
    fn tag_wheels(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) -> Result<i32, AbstractFileError> {
        let mut buf_next = Vec::new();
        let mut wheel_configs = 0_usize;
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"wheelConfiguration" => {
                    wheel_configs += 1;
                    if wheel_configs >= 2 { data.flags.wheels = Capability::Yes }
                    let _ = reader.read_to_end(e.to_end().name());
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        }
        Ok(0)
    }

    // MARK: _sprayTypes
    /// Read spray types
    #[inline]
    fn tag_spray_type(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) -> Result<i32, AbstractFileError> {
        let mut buf_next = Vec::new();
        let mut spray = SprayType::default();

        if let Some(fills) = Self::xml_attribute(e, "fillTypes") {
            spray.fills = Some(fills.split_whitespace().filter_map(|v| if v == "unknown" { None } else { Some(v.to_owned()) }).collect());
        }
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"usageScales" => {
                    spray.width = Self::xml_attribute(&e, "workingWidth").map(|v| v.parse().unwrap_or_default());
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        }
        data.sprays.push(spray);
        Ok(0)
    }

    // MARK: _fillUnit
    /// Read fill unit config
    #[inline]
    fn tag_fill_config(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) -> Result<i32, AbstractFileError> {
        let mut buf_next = Vec::new();
        let mut fill_config = FillConfig::default();

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"fillUnit" => {
                    if Self::xml_attribute(&e, "showInShop") != Some(String::from("false")) {
                        let mut fill_unit = FillUnit::default();
                    
                        if let Some(v) = Self::xml_attribute(&e, "capacity").map(|v|v.parse::<u32>().unwrap_or_default()) {
                            fill_unit.capacity += v;
                        }
                        if let Some(v) = Self::xml_attribute(&e, "fillTypes") {
                            fill_unit.types = v.split_whitespace().filter_map(|v| if v == "unknown" { None } else { Some(v.to_owned()) }).collect();
                        }
                        if let Some(v) = Self::xml_attribute(&e, "fillTypeCategories") {
                            fill_unit.types = v.split_whitespace().map(std::string::ToString::to_string).collect();
                        }
                        fill_config.0.push(fill_unit);
                    }
                    
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        }
        if !fill_config.0.is_empty() { data.fills.push(fill_config); }
        Ok(0)
    }

    // MARK: _specs
    /// Do specs
    #[inline]
    fn tag_specs(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) -> Result<i32, AbstractFileError> {
        let mut buf_next = Vec::new();
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"workingWidth" => data.sorting.working_width = Self::xml_number(&e, reader),
                        b"power" => data.sorting.power = Self::xml_number(&e, reader),
                        b"neededPower" => data.sorting.needed_power = Self::xml_number(&e, reader),
                        b"maxSpeed" => data.sorting.speed_limit = Self::xml_number(&e, reader),
                        _ => (),
                    }
                },
                Ok(Event::Empty(e)) if e.name().as_ref() == b"combination" => {
                    if let Some(filename) = Self::xml_attribute(&e, "xmlFilename") {
                        match Self::unwrap_base_path(filename) {
                            PathType::Base(v) => data.sorting.combos.push(Combo::Base(v)),
                            PathType::Local(v) => data.sorting.combos.push(Combo::Local(v)),
                        }
                    } else if let Some(cat) = Self::xml_attribute(&e, "filterCategory") {
                        data.sorting.combos.push(Combo::Category(cat));
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                Ok(Event::Eof) => return Err(AbstractFileError::XmlParseError),
                _ => (),
            }
        };
        Ok(0) //Self::slurp(e, reader)
    }

}


// MARK: TESTING

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::store_item::StoreItem;
    use crate::files::AbstractFile;

    #[test]
    #[ignore]
    fn valid_vehicle_fill_type() {
        let mut file_handle = AbstractFile::new("tests/test_mods/DETAIL_Samples.zip");
        let actual = StoreItem::from_abstract_file(&mut file_handle, "xml/example-fillunit.xml").expect("process failed");

        
        println!("{}", serde_json::to_string_pretty(&actual).unwrap());

        // let re_read:DescXML = serde_json::from_value(expected).expect("deserialize failed");
        // assert_eq!(re_read, actual);
        // assert_eq!(serde_json::json!(actual.brands), expected);
    }

    #[test]
    fn valid_vehicle_motor() {
        let mut file_handle = AbstractFile::new("tests/test_mods/DETAIL_Samples.zip");
        let actual = StoreItem::from_abstract_file(&mut file_handle, "xml/example-multimotor.xml").expect("process failed");

        
        println!("{}", serde_json::to_string_pretty(&actual).unwrap());

        // let re_read:DescXML = serde_json::from_value(expected).expect("deserialize failed");
        // assert_eq!(re_read, actual);
        // assert_eq!(serde_json::json!(actual.brands), expected);
    }

    #[test]
    #[ignore]
    fn valid_fs25() {
        let mut file_handle = AbstractFile::new("tests/test_mods/FS25_Good.zip");
        let actual = StoreItem::from_abstract_file(&mut file_handle, "fastrac4000.xml").expect("process failed");

        
        println!("{}", serde_json::to_string_pretty(&actual).unwrap());

        // let re_read:DescXML = serde_json::from_value(expected).expect("deserialize failed");
        // assert_eq!(re_read, actual);
        // assert_eq!(serde_json::json!(actual.brands), expected);
    }

    #[test]
    #[ignore]
    fn valid_sprays() {
        let mut file_handle = AbstractFile::new("tests/test_mods/DETAIL_Samples.zip");
        let actual = StoreItem::from_abstract_file(&mut file_handle, "xml/example-spraytypes.xml").expect("process failed");

        
        println!("{}", serde_json::to_string_pretty(&actual).unwrap());

        // let re_read:DescXML = serde_json::from_value(expected).expect("deserialize failed");
        // assert_eq!(re_read, actual);
        // assert_eq!(serde_json::json!(actual.brands), expected);
    }
}