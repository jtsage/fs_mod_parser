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
    pub placeables : Vec<ModDetailPlace>,
    pub vehicles   : Vec<ModDetailVehicle>,
}

impl ModDetail {
    #[must_use]
    pub fn new() -> Self {
        ModDetail {
            brands     : HashMap::new(),
            issues     : HashSet::new(),
            l10n       : HashMap::new(),
            placeables : vec![],
            vehicles   : vec![],
        }
    }

    #[must_use]
    pub fn fast_fail(e : ModDetailError) -> Self {
        let mut record = ModDetail::new();
        record.add_issue(e);
        record
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
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or("{}".to_string())
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        self.to_string()
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
    pub beacons   : VehicleCapability,
    pub color     : VehicleCapability,
    pub enterable : VehicleCapability,
    pub lights    : VehicleCapability,
    pub motorized : VehicleCapability,
    pub wheels    : VehicleCapability,
}

impl ModDetailVehicleFlags {
    fn new() -> Self {
        ModDetailVehicleFlags {
            beacons   : VehicleCapability::No,
            color     : VehicleCapability::No,
            enterable : VehicleCapability::No,
            lights    : VehicleCapability::No,
            motorized : VehicleCapability::No,
            wheels    : VehicleCapability::No
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
    pub fuel_type         : Option<String>,
    pub transmission_type : Option<String>,
    pub motors            : Vec<MotorEntry>,
}

impl ModDetailVehicleEngine {
    fn new() -> Self {
        ModDetailVehicleEngine {
            fuel_type         : None,
            transmission_type : None,
            motors            : vec![],
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailSprayType {
    pub fills : Vec<String>,
    pub width : Option<u32>,
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
    pub functions      : Vec<String>,
    pub joint_accepts  : Vec<String>,
    pub joint_requires : Vec<String>,
    pub name           : String,
    pub price          : u32,
    pub specs          : HashMap<String, u32>,
    pub weight         : u32,
}

impl ModDetailVehicleSpecs {
    fn new() -> Self {
        ModDetailVehicleSpecs {
            functions      : vec![],
            joint_accepts  : vec![],
            joint_requires : vec![],
            name           : String::new(),
            price          : 0,
            specs          : HashMap::new(),
            weight         : 0,
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
    #[must_use]
    pub fn new() -> Self {
        ModDetailVehicle {
            fill_spray  : ModDetailVehicleFillSpray::new(),
            flags       : ModDetailVehicleFlags::new(),
            icon_base   : None,
            icon_file   : None,
            master_type : String::from("vehicle"),
            motor       : ModDetailVehicleEngine::new(),
            sorting     : ModDetailVehicleSorting::new(),
            specs       : ModDetailVehicleSpecs::new(),
        }
    }
}

impl Default for ModDetailVehicle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorValue {
    pub rpm : u32,
    pub value : u32,
}
impl MotorValue {
    #[must_use]
    pub fn new(rpm: f32, value : f32) -> Self {
        MotorValue {
            rpm   : Self::round_to_u32(rpm),
            value : Self::round_to_u32(value)
        }
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn round_to_u32(num:f32) -> u32 {
        num.round() as u32
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorEntry {
    pub name        : String,
    pub horse_power : Vec<MotorValue>,
    pub max_speed   : u32,
    pub speed_kph   : Vec<MotorValue>,
    pub speed_mph   : Vec<MotorValue>,
}

impl MotorEntry {
    #[must_use]
    pub fn new(name : String, max_speed : u32) -> Self {
        MotorEntry {
            name,
            horse_power : vec![],
            max_speed,
            speed_kph : vec![],
            speed_mph : vec![],
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlaceSorting {
    pub category        : Option<String>,
    pub functions       : Vec<String>,
    pub has_color       : VehicleCapability,
    pub income_per_hour : u32,
    pub name            : Option<String>,
    pub price           : u32,
    pub type_name       : Option<String>,
}

impl ModDetailPlaceSorting {
    fn new() -> Self {
        ModDetailPlaceSorting {
            category        : None,
            functions       : vec![],
            has_color       : VehicleCapability::No,
            income_per_hour : 0,
            name            : None,
            price           : 0,
            type_name       : None
        }
    }
    
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlaceAnimals {
    pub beehive_exists    : bool,
    pub beehive_per_day   : u32,
    pub beehive_radius    : u32,
    pub husbandry_animals : u32,
    pub husbandry_exists  : bool,
    pub husbandry_type    : Option<String>,
}

impl ModDetailPlaceAnimals {
    fn new() -> Self {
        ModDetailPlaceAnimals {
            beehive_exists    : false,
            beehive_per_day   : 0,
            beehive_radius    : 0,
            husbandry_animals : 0,
            husbandry_exists  : false,
            husbandry_type    : None
        }
    }
    
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlaceStorage {
    pub objects         : Option<u32>,
    pub silo_capacity   : u32,
    pub silo_exists     : bool,
    pub silo_fill_cats  : Vec<String>,
    pub silo_fill_types : Vec<String>,
}

impl ModDetailPlaceStorage {
    fn new() -> Self {
        ModDetailPlaceStorage {
            objects         : None,
            silo_capacity   : 0,
            silo_exists     : false,
            silo_fill_cats  : vec![],
            silo_fill_types : vec![],
        }
    }
}

pub type ProductionIngredients = Vec<ProductionIngredient>;
pub type ProductionRecipe = Vec<ProductionIngredients>;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionIngredient {
    amount : f32,
    fill_type : String
}
impl ProductionIngredient {
    #[must_use]
    pub fn new(fill_type: String, amount: f32) -> Self {
        ProductionIngredient {
            amount,
            fill_type,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionBoost {
    amount       : f32,
    boost_factor : f32,
    fill_type    : String,
}
impl ProductionBoost {
    #[must_use]
    pub fn new(fill_type: String, amount: f32, boost_factor: f32) -> Self {
        ProductionBoost {
            amount,
            boost_factor,
            fill_type,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailProduction {
    pub boosts          : Vec<ProductionBoost>,
    pub cost_per_hour   : u32,
    pub cycles_per_hour : u32,
    pub name            : String,
    pub output          : Vec<ProductionIngredient>,
    pub params          : String,
    pub recipe          : ProductionRecipe,
}

impl ModDetailProduction {
    #[must_use]
    pub fn new() -> Self {
        ModDetailProduction {
            boosts          : vec![],
            cost_per_hour   : 1,
            cycles_per_hour : 1,
            name            : String::from("--"),
            output          : vec![],
            params          : String::new(),
            recipe          : vec![]
        }
    }
}

impl Default for ModDetailProduction {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlace {
    pub animals     : ModDetailPlaceAnimals,
    pub icon_base   : Option<String>,
    pub icon_file   : Option<String>,
    pub master_type : String,
    pub productions : Vec<ModDetailProduction>,
    pub sorting     : ModDetailPlaceSorting,
    pub storage     : ModDetailPlaceStorage,
}

impl ModDetailPlace {
    #[must_use]
    pub fn new() -> Self {
        ModDetailPlace {
            animals     : ModDetailPlaceAnimals::new(),
            icon_base   : None,
            icon_file   : None,
            master_type : String::from("placeable"),
            productions : vec![],
            sorting     : ModDetailPlaceSorting::new(),
            storage     : ModDetailPlaceStorage::new()
        }
    }
}

impl Default for ModDetailPlace {
    fn default() -> Self {
        Self::new()
    }
}