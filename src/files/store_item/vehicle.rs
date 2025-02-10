use std::f32::consts::PI;

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
    pub icon_base: Option<String>,
    /// path to local icon
    pub icon_file: Option<String>,
    /// icon data
    pub icon_data: Option<String>,
    // motor information
    pub motor: DriveTrain,
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
pub type FillConfig = Vec<FillUnit>;

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
/// Vehicle engine sub-record
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DriveTrain {
    /// fuel type
    pub fuel_type: Option<String>,
    /// transmission type (primary)
    pub transmission_type: Option<String>,
    /// motor configurations
    pub motors: Vec<Motor>,
}


/// motor definition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Motor {
    /// name of motor
    pub name: String,
    /// name of transmission
    pub transmission : Option<String>,
    /// list of rpm->hp values
    pub horse_power: Vec<MotorValue>,
    /// maximum stated speed (from author)
    pub max_speed: u32,
    /// list of rpm->kph values
    pub speed_kph: Vec<MotorValue>,
    /// list of rpm->mph values
    pub speed_mph: Vec<MotorValue>,
}

pub type MotorValue = (u32, u32);

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
                    let filename = Self::unwrap_base_path(filename);
                    match &filename {
                        PathType::Base(_) => data.icon_base = Some(filename.clone().to_dds()),
                        PathType::Local(_) => data.icon_file = Some(filename.clone().to_dds()),
                    }
                }
                Ok(0)
            },
            (b"parentFile", 1) => {
                data.parent_item = Self::xml_attribute(e, "xmlFilename");
                Self::slurp(e, reader)
            },

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
            (b"specs", 2) => { Self::tag_specs(reader, data, e); Ok(0) },

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
            (b"wheels", 1) => { Self::tag_wheels(reader, data, e); Ok(0) },
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

            // MARK: ~fill/spray/motor
            (b"sprayType", 3) => { Self::tag_spray_type(reader, data, e); Ok(0) },
            (b"fillUnitConfiguration", 3) => { Self::tag_fill_config(reader, data, e); Ok(0) },
            (b"motorConfigurations", 2) => { Self::tag_motors(reader, data, e); Ok(0) },

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
            (b"consumer", _) if data.motor.fuel_type.is_none() => {
                data.motor.fuel_type = Self::xml_attribute(e, "fillType");
            }
            _ => (),
        }
    }
}

impl Vehicle {
    // MARK: _wheels
    /// Do wheels
    #[inline]
    fn tag_wheels(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) {
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
                _ => (),
            }
        }
    }

    // MARK: _sprayTypes
    /// Read spray types
    #[inline]
    fn tag_spray_type(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) {
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
                _ => (),
            }
        }
        data.sprays.push(spray);
    }

    // MARK: _fillUnit
    /// Read fill unit config
    #[inline]
    fn tag_fill_config(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) {
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
                            fill_unit.categories = v.split_whitespace().map(std::string::ToString::to_string).collect();
                        }
                        fill_config.push(fill_unit);
                    }
                    
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }
        if !fill_config.is_empty() { data.fills.push(fill_config); }
    }

    // MARK: _specs
    /// Do specs
    #[inline]
    fn tag_specs(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) {
        let mut buf_next = Vec::new();
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"workingWidth" => data.sorting.working_width = Self::xml_number(&e, reader),
                        b"power"        => data.sorting.power = Self::xml_number(&e, reader),
                        b"neededPower"  => data.sorting.needed_power = Self::xml_number(&e, reader),
                        b"maxSpeed"     => data.sorting.speed_limit = Self::xml_number(&e, reader),
                        _ => (),
                    }
                },
                Ok(Event::Empty(e)) if e.name().as_ref() == b"combination" => {
                    if let Some(filename) = Self::xml_attribute(&e, "xmlFilename") {
                        match Self::unwrap_base_path(filename) {
                            PathType::Base(v)  => data.sorting.combos.push(Combo::Base(v)),
                            PathType::Local(v) => data.sorting.combos.push(Combo::Local(v)),
                        }
                    } else if let Some(cat) = Self::xml_attribute(&e, "filterCategory") {
                        data.sorting.combos.push(Combo::Category(cat));
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        };
    }

   
    // MARK: _motors
    /// Do motors
    #[inline]
    fn tag_motors(reader: &mut Reader<&[u8]>, data: &mut Self, e : &BytesStart) {
        let mut buf_next = Vec::new();
        let mut motor = MotorBuild::new();
        let mut next_motor = true;

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e) | Event::Empty(e)) => {
                    match e.name().as_ref() {
                        b"motorConfiguration" => {
                            if let Some(name) = Self::xml_attribute(&e, "name") {
                                motor.name = name;
                            }
                            if let Some(hp) = Self::xml_attribute_number(&e, "hp") {
                                motor.declared_hp = hp;
                            }
                        },
                        b"motor" => {
                            motor.torque_scale = Self::xml_attribute_number(&e, "torqueScale").unwrap_or(1_f32);
                            if let Some(rpm) = Self::xml_attribute_number(&e, "maxRpm") {
                                motor.rpm = rpm;
                            }
                            if let Some(speed) = Self::xml_attribute_number(&e, "maxForwardSpeed") {
                                motor.declared_speed = speed;
                            }
                        },
                        b"torque" => {
                            if next_motor { next_motor = false; motor.torque.clear(); }

                            let norm_rpm = Self::xml_attribute_number(&e, "normRpm").unwrap_or(1_f32);
                            let torque = Self::xml_attribute_number(&e, "torque").unwrap_or(1_f32);

                            if let Some(rpm) = Self::xml_attribute_number(&e, "rpm") {
                                motor.torque.push((rpm, torque));
                            } else {
                                motor.torque.push((motor.rpm * norm_rpm, torque));
                            }
                        },
                        b"transmission" => {
                            if let Some(name) = Self::xml_attribute(&e, "name") {
                                if data.motor.transmission_type.is_none() {
                                    data.motor.transmission_type = Some(name.clone());
                                }
                                motor.trans = name;
                            }

                            // New transmission declared, reset ratio
                            motor.min_fwd_gear_and_axle_ratio = f32::MAX;

                            // Axel ratio override
                            if let Some(ratio) = Self::xml_attribute_number(&e, "axleRatio") {
                                motor.axle_ratio = ratio;
                            }

                            // Ratio is directly defined, load and apply it
                            if let Some(scale) = Self::xml_attribute_number::<f32>(&e, "minForwardGearRatio") {
                                motor.min_fwd_gear_and_axle_ratio = scale * motor.axle_ratio;
                            }
                        },
                        b"forwardGear" => {
                            // Ratio not defined on transmission, calculate from each gear maxSpeed
                            if let Some(speed) = Self::xml_attribute_number::<f32>(&e, "maxSpeed") {
                                motor.min_fwd_gear_and_axle_ratio = f32::min(
                                    motor.min_fwd_gear_and_axle_ratio,
                                    motor.axle_ratio * (motor.rpm * PI / (speed / 3.6_f32 * 30_f32)),
                                );
                            }
                            // Ratio not defined on transmission, calculate from each gear ratio
                            if let Some(gear) = Self::xml_attribute_number::<f32>(&e, "gearRatio") {
                                motor.min_fwd_gear_and_axle_ratio = f32::min(
                                    motor.min_fwd_gear_and_axle_ratio,
                                    motor.axle_ratio * gear,
                                );
                            }
                        },
                        _ => (),
                    }
                },
                Ok(Event::End(e)) if e.name().as_ref() == b"motorConfiguration" => {
                    next_motor = true;
                    data.motor.motors.push(motor.build());
                    motor.reset();
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }
    }
}

/// Torque entry (calculated rpm, torque)
type Torque = (f32, f32);

/// Motor builder
#[derive(Default, Debug)]
struct MotorBuild {
    /// Max RPM
    pub rpm: f32,
    /// Transmission name
    pub trans : String,
    /// Axel ratio for calculations
    pub min_fwd_gear_and_axle_ratio: f32,
    /// Axel ratio from XML
    pub axle_ratio: f32,
    /// Motor name
    pub name : String,
    /// Speed given in XML
    pub declared_speed : u32,
    /// HP given in XML
    pub declared_hp : u32,
    /// List of torque settings
    pub torque : Vec<Torque>,
    /// Torque scale for motor
    pub torque_scale : f32,
}

impl MotorBuild {
    /// Create a new motor builder
    fn new() -> Self {
        Self {
            rpm : 1800_f32,
            min_fwd_gear_and_axle_ratio : f32::MAX,
            axle_ratio: 1_f32,
            torque_scale : 1_f32,
            ..Default::default()
        }
    }
    /// Reset those value that must be in a new definition
    fn reset(&mut self) {
        self.name = String::new();
        self.axle_ratio = 1_f32;
        self.torque_scale = 1_f32;
    }
    /// Build a [`Motor`] from the builder
    fn build(&self) -> Motor {
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let horse_power:Vec<MotorValue> = self.torque.clone().into_iter().map(|v| (
            v.0.round() as u32,
            (self.torque_scale * (1.359_621_6 * PI * v.0 * v.1) / 30.0).round() as u32
        )).collect();

        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let kph:Vec<MotorValue> = self.torque.clone().into_iter().map(|v| (
            v.0.round() as u32,
            (3.6 * ((v.0 * PI) / (30.0 * self.min_fwd_gear_and_axle_ratio))).round() as u32
        )).collect();

        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mph:Vec<MotorValue> = self.torque.clone().into_iter().map(|v| (
            v.0.round() as u32,
            (3.6 * ((v.0 * PI) / (30.0 * self.min_fwd_gear_and_axle_ratio) * 0.621_371)).round() as u32
        )).collect();

        Motor {
            name: format!("{} {}hp", self.name, self.declared_hp),
            transmission: if self.trans.is_empty() { None } else { Some(self.trans.clone()) },
            horse_power,
            max_speed: self.declared_speed,
            speed_kph : kph,
            speed_mph : mph
        }
    }
}

// MARK: TESTING

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::store_item::StoreItem;
    use crate::files::AbstractFile;

    fn xml_test(str: &str) -> String {
        format!("<?xml version=\"1.0\" ?>\n{str}")
    }

    #[test]
    fn icon_mapping() {
    let xml = xml_test(r#"
        <vehicle>
            <storeData><image>$data/path/to/data/file.png</image></storeData>
        </vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        assert_eq!(vehicle.icon_base, Some(String::from("path/to/data/file.dds")));
        assert!(vehicle.icon_file.is_none());

        let xml = xml_test(r#"
            <vehicle>
                <storeData><image>local/path/to/data/file.png</image></storeData>
            </vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        assert_eq!(vehicle.icon_file, Some(String::from("local/path/to/data/file.dds")));
        assert!(vehicle.icon_base.is_none());

        let xml = xml_test(r#"
            <vehicle>
                <storeData><image></image></storeData>
            </vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        assert_eq!(vehicle.icon_file, Some(String::from("")));
        assert!(vehicle.icon_base.is_none());
    }

    #[test]
    fn spray_types() {
        let xml = xml_test(r#"
        <vehicle><sprayer><sprayTypes>
            <sprayType foldingConfigurationIndex="1" fillTypes="fertilizer unknown">
                <usageScales workingWidth="10" />
            </sprayType>
            <sprayType foldingConfigurationIndex="1" fillTypes="lime">
                <usageScales workingWidth="20" />
            </sprayType>
        </sprayTypes></sprayer></vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        let expected = vec![
            SprayType { fills : Some(vec![String::from("fertilizer")]), width : Some(10.0) },
            SprayType { fills : Some(vec![String::from("lime")]), width : Some(20.0) },
        ];

        assert_eq!(vehicle.sprays, expected);
    }

    #[test]
    fn spray_types_failures() {
        let xml = xml_test(r#"
        <vehicle><sprayer><sprayTypes>
            <garbageTag></garbageTag>
            <sprayType foldingConfigurationIndex="1" fillTypes="lime">
                <usageScales workingWidth="20" />"#);
        let actual = StoreItem::from_string(xml.as_ref());
        
        assert_eq!(actual, Err(AbstractFileError::XmlParseError));
    }

    #[test]
    fn new_style_color() {
        let xml = xml_test(r#"
            <vehicle>
                <baseColorConfigurations useDefaultColors="true" defaultColorMaterialTemplateName="calibratedGlossPaint">
                    <baseColorConfiguration color="JCB_YELLOW1" />
                </baseColorConfigurations>
                <designColorConfigurations useDefaultColors="true" defaultColorMaterialTemplateName="calibratedGlossPaint" title="$l10n_configuration_grillColor">
                    <designColorConfiguration color="JCB_YELLOW1"/>
                </designColorConfigurations>
            </vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        assert_eq!(vehicle.flags.color, Capability::Yes);
    }

    #[test]
    fn fill_types() {
        let xml = xml_test(r#"
        <vehicle><fillUnit><fillUnitConfigurations>
            <fillUnitConfiguration>
                <fillUnits>
                    <fillUnit fillTypes="fertilizer lime" capacity="15000"></fillUnit>
                </fillUnits>
            </fillUnitConfiguration>
            <fillUnitConfiguration>
                <fillUnits>
                    <fillUnit fillTypes="fertilizer" capacity="8000"></fillUnit>
                    <fillUnit fillTypes="seeds" capacity="3000"></fillUnit>
                </fillUnits>
            </fillUnitConfiguration>
            <fillUnitConfiguration>
                <fillUnits>
                    <fillUnit fillTypeCategories="SHOVEL" capacity="1000" ></fillUnit>
                    <fillUnit fillTypeCategories="SHOVEL" capacity="10000" showInShop="false" showOnHud="false"/>
                </fillUnits>
            </fillUnitConfiguration>
        </fillUnitConfigurations></fillUnit></vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        let expected = vec![
            vec![
                FillUnit{ categories : vec![], types : vec![String::from("fertilizer"), String::from("lime")], capacity : 15000 },
            ],
            vec![
                FillUnit{ categories : vec![], types : vec![String::from("fertilizer")], capacity : 8000 },
                FillUnit{ categories : vec![], types : vec![String::from("seeds")], capacity : 3000 },
            ],
            vec![
                FillUnit{ categories : vec![String::from("SHOVEL")], types : vec![], capacity : 1000 },
            ],
        ];

        assert_eq!(vehicle.fills, expected);
    }

    #[test]
    fn vehicle_motor_trans_min_fwd() {
        let xml = xml_test(r#"
        <vehicle><motorized><motorConfigurations>
            <motorConfiguration name="8RX 310 Electric" hp="357" price="0" consumerConfigurationIndex="1">
                <motor torqueScale="1.507" minRpm="900" maxRpm="2200" maxForwardSpeed="42" maxBackwardSpeed="20" brakeForce="3.5" lowBrakeForceScale="0.33" dampingRateScale="0.25">
                    <torque normRpm="0.45" torque="0.9"/>
                    <torque normRpm="0.5" torque="0.97"/>
                    <torque normRpm="0.59" torque="1"/>
                    <torque normRpm="0.72" torque="1"/>
                    <torque normRpm="0.86" torque="0.88"/>
                    <torque normRpm="1" torque="0.72"/>
                </motor>
                <transmission minForwardGearRatio="17" maxForwardGearRatio="310" minBackwardGearRatio="32" maxBackwardGearRatio="310" name="$l10n_info_transmission_cvt"/>
                <objectChange node="engineConfig310_decal" visibilityActive="true" visibilityInactive="false" />
            </motorConfiguration>
        </motorConfigurations></motorized></vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        let expected = DriveTrain {
            fuel_type : None,
            transmission_type : Some(String::from("$l10n_info_transmission_cvt")),
            motors : vec![
                Motor {
                    name:String::from("8RX 310 Electric 357hp"),
                    transmission: Some(String::from("$l10n_info_transmission_cvt")),
                    horse_power: vec![
                        (990, 191), (1100, 229), (1298, 279),
                        (1584, 340), (1892, 357), (2200, 340),
                    ],
                    max_speed: 42,
                    speed_kph: vec![
                        (990, 22), (1100, 24), (1298, 29),
                        (1584, 35), (1892, 42), (2200, 49),
                    ],
                    speed_mph: vec![
                        (990, 14), (1100, 15), (1298, 18),
                        (1584, 22), (1892, 26), (2200, 30),
                    ]
                }
            ]
        };

        assert_eq!(vehicle.motor, expected);
    }

    #[test]
    fn vehicle_motor_trans_gear_ratio() {
        let xml = xml_test(r#"
        <vehicle><motorized><motorConfigurations>
            <motorConfiguration name="Pickup 2017" hp="300" price="0">
                <motor torqueScale="0.6" minRpm="1000" maxRpm="6000" maxForwardSpeed="120" maxBackwardSpeed="22" brakeForce="2.2" lowBrakeForceScale="0.22" dampingRateScale="0.4">
                    <torque rpm="1000" torque="0.9"/>
                    <torque rpm="2400" torque="1"/>
                    <torque rpm="3480" torque="1"/>
                    <torque rpm="4560" torque="0.75"/>
                    <torque rpm="5280" torque="0.63"/>
                    <torque rpm="6000" torque="0.2"/>
                </motor>
                <transmission autoGearChangeTime="1" gearChangeTime="0.4" name="$l10n_info_transmission_manual" axleRatio="25" startGearThreshold="0.3">
                    <directionChange useGear="true"/>
                    <backwardGear gearRatio="4.066" name="R"/>
                    <forwardGear gearRatio="4.784"/>
                    <forwardGear gearRatio="2.423"/>
                    <forwardGear gearRatio="1.443"/>
                    <forwardGear gearRatio="1.000"/>
                    <forwardGear gearRatio="0.826"/>
                    <forwardGear gearRatio="0.643"/>
                </transmission>
            </motorConfiguration>
        </motorConfigurations>
        <consumerConfigurations>
            <consumerConfiguration consumersEmptyWarning="$l10n_warning_motorBatteryEmpty">
                <consumer fillUnitIndex="1" usage="107" fillType="electricCharge" />
            </consumerConfiguration>
        </consumerConfigurations></motorized></vehicle>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let vehicle = actual.vehicle.unwrap();

        let expected = DriveTrain {
            fuel_type : Some(String::from("electricCharge")),
            transmission_type : Some(String::from("$l10n_info_transmission_manual")),
            motors : vec![
                Motor {
                    name:String::from("Pickup 2017 300hp"),
                    transmission: Some(String::from("$l10n_info_transmission_manual")),
                    horse_power: vec![
                        (1000, 77), (2400, 205), (3480, 297),
                        (4560, 292), (5280, 284), (6000, 103),
                    ],
                    max_speed: 120,
                    speed_kph: vec![
                        (1000, 23), (2400, 56), (3480, 82),
                        (4560, 107), (5280, 124), (6000, 141),
                    ],
                    speed_mph: vec![
                        (1000, 15), (2400, 35), (3480, 51),
                        (4560, 66), (5280, 77), (6000, 87),
                    ]
                }
            ]
        };

        assert_eq!(vehicle.motor, expected);
    }

    #[test]
    fn from_file_fill_unit() {
        let filename = "tests/test_mods/DETAIL_Samples.zip";
        let item = "xml/example-fill-unit.xml";
        let json = "json/example-fill-unit.json";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = StoreItem::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = file_handle.text(json).unwrap();
        let re_read:StoreItem = serde_json::from_str(&expected).unwrap();

        assert_eq!(actual, re_read);
    }

    #[test]
    fn from_file_motors() {
        let filename = "tests/test_mods/DETAIL_Samples.zip";
        let item = "xml/example-multi-motor.xml";
        let json = "json/example-multi-motor.json";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = StoreItem::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = file_handle.text(json).unwrap();
        let re_read:StoreItem = serde_json::from_str(&expected).unwrap();

        assert_eq!(actual, re_read);
    }

    #[test]
    fn from_file_spray_types() {
        let filename = "tests/test_mods/DETAIL_Samples.zip";
        let item = "xml/example-spray-types.xml";
        let json = "json/example-spray-types.json";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = StoreItem::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = file_handle.text(json).unwrap();
        let re_read:StoreItem = serde_json::from_str(&expected).unwrap();

        assert_eq!(actual, re_read);
    }

    #[test]
    fn from_file_parent() {
        let filename = "tests/test_mods/DETAIL_Samples.zip";
        let item = "xml/vehicle-with-parent.xml";
        let json = "json/vehicle-with-parent.json";
        let dump = true;

        let mut file_handle = AbstractFile::new(filename);
        let actual = StoreItem::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = file_handle.text(json).unwrap();
        let re_read:StoreItem = serde_json::from_str(&expected).unwrap();

        assert_eq!(actual, re_read);
    }
}