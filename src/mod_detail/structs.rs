use std::collections::{HashMap, HashSet};
use serde::ser::{Serialize, Serializer};

#[derive(serde::Serialize, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum ModDetailError {
    FileReadFail,
    NotModModDesc,
    BrandMissingIcon,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetail {
    #[serde(skip_serializing)]
    pub brands     : BrandDefinition,
    pub issues     : HashSet<ModDetailError>,
    #[serde(skip_serializing)]
    pub l10n       : LanguageDefinition,
    pub placeables : String,
    pub vehicles   : Vec<ModDetailVehicle>,
}

impl ModDetail {
    pub fn new() -> Self {
        ModDetail {
            brands     : HashMap::new(),
            issues     : HashSet::new(),
            l10n       : HashMap::new(),
            placeables : String::new(),
            vehicles   : vec![],
        }
    }

    pub fn add_issue(&mut self, issue : ModDetailError) -> &mut Self {
        self.issues.insert(issue);
        self
    }
    pub fn add_lang(&mut self, language : &str, key : &str, value : &str) -> &mut Self{
        let this_language = self.l10n.entry(language.to_string()).or_default();
    
        this_language.insert(key.to_string().to_lowercase(), value.to_string());

        self
        
    }
    pub fn add_brand(&mut self, key_name : &str, title: Option<&str>) -> &mut ModDetailBrand{
        let this_brand = self.brands.entry(key_name.to_string()).or_default();

        this_brand.title = match title {
            Some(title) => title.to_string(),
            None => key_name.to_string()
        };
        this_brand
    }
    pub fn pretty_print(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or("{}".to_string())
    }
}

impl Default for ModDetail {
    fn default() -> Self {
        ModDetail::new()
    }
}

impl std::fmt::Display for ModDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}
type LanguageDefinition = HashMap<String, HashMap<String, String>>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailBrand {
    pub title : String,
    pub icon_file : Option<String>,
    pub icon_base : Option<String>
}

impl ModDetailBrand {
    fn new() -> Self {
        ModDetailBrand { title: String::new(), icon_file: None, icon_base: None }
    }
}
impl Default for ModDetailBrand {
    fn default() -> Self {
        ModDetailBrand::new()
    }
}

type BrandDefinition = HashMap<String, ModDetailBrand>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleSorting {
    pub brand            : Option<String>,
    pub category         : Option<String>,
    pub combos           : Vec<String>,
    pub name             : Option<String>,
    pub type_name        : Option<String>,
    pub type_description : Option<String>,
    pub year             : Option<u32>,
}

impl ModDetailVehicleSorting {
    fn new() -> Self {
        ModDetailVehicleSorting {
            brand            : None,
            category         : None,
            combos           : vec![],
            name             : None,
            type_name        : None,
            type_description : None,
            year             : None,
        }
    }
}

pub enum VehicleCapability {
    Yes,
    No
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleFlags {
    pub enterable : VehicleCapability,
    pub motorized : VehicleCapability,
    pub beacons   : VehicleCapability,
    pub color     : VehicleCapability,
    pub lights    : VehicleCapability,
    pub wheels    : VehicleCapability,
}

impl ModDetailVehicleFlags {
    fn new() -> Self {
        ModDetailVehicleFlags {
            enterable: VehicleCapability::No,
            motorized: VehicleCapability::No,
            beacons: VehicleCapability::No,
            color: VehicleCapability::No,
            lights: VehicleCapability::No,
            wheels: VehicleCapability::No
        }
    }
}

impl Serialize for VehicleCapability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            VehicleCapability::Yes => serializer.serialize_bool(true),
            VehicleCapability::No  => serializer.serialize_bool(false),
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleEngine {
    fuel_type         : Option<String>,
    transmission_type : Option<String>,
    mph               : Option<String>,
    kph               : Option<String>,
    hp                : Option<String>
}

impl ModDetailVehicleEngine {
    fn new() -> Self {
        ModDetailVehicleEngine {
            fuel_type         : None,
            transmission_type : None,
            mph               : None,
            kph               : None,
            hp                : None,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailSprayType {
    pub fills : Vec<String>,
    pub width : u32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleFillSpray {
    pub fill_cat    : Vec<String>,
    pub fill_level  : u32,
    pub fill_type   : Vec<String>,
    pub spray_types : Vec<ModDetailSprayType>,
}

impl ModDetailVehicleFillSpray {
    fn new() -> Self {
        ModDetailVehicleFillSpray {
            fill_cat    : vec![],
            fill_level  : 0,
            fill_type   : vec![],
            spray_types : vec![],
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleSpecs {
    pub functions     : Vec<String>,
    pub joint_can_use : Vec<String>,
    pub joint_need    : Vec<String>,
    pub name          : String,
    pub price         : u32,
    pub specs         : HashMap<String, u32>,
    pub weight        : u32,
}

impl ModDetailVehicleSpecs {
    fn new() -> Self {
        ModDetailVehicleSpecs {
            functions     : vec![],
            joint_can_use : vec![],
            joint_need    : vec![],
            name          : String::new(),
            price         : 0,
            specs         : HashMap::new(),
            weight        : 0,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicle {
    pub fill_spray  : ModDetailVehicleFillSpray,
    pub flags       : ModDetailVehicleFlags,
    pub icon_base   : Option<String>,
    pub icon_file   : Option<String>,
    pub master_type : String,
    pub motor       : ModDetailVehicleEngine,
    pub sorting     : ModDetailVehicleSorting,
    pub specs       : ModDetailVehicleSpecs,
}

impl ModDetailVehicle {
    pub fn new() -> Self {
        ModDetailVehicle {
            fill_spray  : ModDetailVehicleFillSpray::new(),
            flags       : ModDetailVehicleFlags::new(),
            icon_base   : None,
            icon_file   : None,
            master_type : String::new(),
            motor       : ModDetailVehicleEngine::new(),
            sorting     : ModDetailVehicleSorting::new(),
            specs       : ModDetailVehicleSpecs::new(),
        }
    }

    pub fn pretty_print(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or("{}".to_string())
    }
}

impl Default for ModDetailVehicle {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ModDetailVehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}