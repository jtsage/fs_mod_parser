//! Mod Detail data structures
use serde::ser::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};

/// Detail errors
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum ModDetailError {
    /// Could not read file
    FileReadFail,
    /// modDesc.xml missing
    NotModModDesc,
    /// Brand icon is missing
    BrandMissingIcon,
    /// Bad storeItem Record
    StoreItemMissing,
    /// Bad storeItem XML
    StoreItemBroken,
}

impl Serialize for ModDetailError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ModDetailError::FileReadFail => {
                serializer.serialize_unit_variant("ModDetailError", 0, "DETAIL_ERROR_UNREADABLE")
            }
            ModDetailError::NotModModDesc => serializer.serialize_unit_variant(
                "ModDetailError",
                1,
                "DETAIL_ERROR_MISSING_MODDESC",
            ),
            ModDetailError::BrandMissingIcon => {
                serializer.serialize_unit_variant("ModDetailError", 2, "DETAIL_ERROR_MISSING_ICON")
            }
            ModDetailError::StoreItemMissing => {
                serializer.serialize_unit_variant("ModDetailError", 3, "DETAIL_ERROR_MISSING_ITEM")
            }
            ModDetailError::StoreItemBroken => {
                serializer.serialize_unit_variant("ModDetailError", 4, "DETAIL_ERROR_PARSE_ITEM")
            }
        }
    }
}

/// Mod Detail Data
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetail {
    /// list of brands
    pub brands: BrandDefinition,
    /// list of detected issues
    pub issues: HashSet<ModDetailError>,
    /// Item brands
    pub item_brands: HashSet<String>,
    /// Item categories
    pub item_categories: HashSet<String>,
    /// l10n languages, keys, and strings
    pub l10n: LanguageDefinition,
    /// placables
    pub placeables: HashMap<String, ModDetailPlace>,
    /// vehicles
    pub vehicles: HashMap<String, ModDetailVehicle>,
}

impl ModDetail {
    /// Create new mod detail record
    #[must_use]
    pub fn new() -> Self {
        ModDetail {
            brands: HashMap::new(),
            issues: HashSet::new(),
            item_brands: HashSet::new(),
            item_categories: HashSet::new(),
            l10n: HashMap::new(),
            placeables: HashMap::new(),
            vehicles: HashMap::new(),
        }
    }

    /// Create new mod detail record with a single error condition
    #[must_use]
    pub fn fast_fail(e: ModDetailError) -> Self {
        let mut record = ModDetail::default();
        record.add_issue(e);
        record
    }

    /// Add an error to a mod detail record
    pub fn add_issue(&mut self, issue: ModDetailError) -> &mut Self {
        self.issues.insert(issue);
        self
    }

    /// Add (or alter) a language code with a new key and string
    pub fn add_lang(&mut self, language: &str, key: &str, value: &str) -> &mut Self {
        let this_language = self.l10n.entry(language.to_owned()).or_default();

        this_language.insert(key.to_owned().to_lowercase(), value.to_owned());

        self
    }

    /// Add a brand record
    pub fn add_brand(&mut self, key_name: &str, title: Option<&str>) -> &mut ModDetailBrand {
        let this_brand = self.brands.entry(key_name.to_owned()).or_default();

        this_brand.title = match title {
            Some(title) => title.to_owned(),
            None => key_name.to_owned(),
        };
        this_brand
    }

    /// Output as pretty-print JSON
    #[must_use]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or(String::from("{}"))
    }

    /// Output as JSON
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
        f.write_str(&serde_json::to_string(&self).unwrap_or(String::from("{}")))
    }
}

/// Nested language definition langCode => [key, string]
type LanguageDefinition = HashMap<String, HashMap<String, String>>;

/// Added brand
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailBrand {
    /// name of the brand (human readable)
    pub title: String,
    /// icon file, if read and included
    pub icon_file: Option<String>,
    /// icon path, if it references the base game
    pub icon_base: Option<String>,
    /// icon original entry
    pub icon_orig: Option<String>
}

impl ModDetailBrand {
    /// Create new brand record
    fn new() -> Self {
        ModDetailBrand {
            title: String::new(),
            icon_file: None,
            icon_base: None,
            icon_orig: None,
        }
    }
}
impl Default for ModDetailBrand {
    fn default() -> Self {
        ModDetailBrand::new()
    }
}

/// Brand definition mapping Brand Key -> Brand Record
type BrandDefinition = HashMap<String, ModDetailBrand>;

/// Vehicle sorting data
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleSorting {
    /// brand KEY
    pub brand: Option<String>,
    /// category
    pub category: Option<String>,
    /// list of combos (local or basegame)
    pub combos: Vec<String>,
    /// name of vehicle
    pub name: Option<String>,
    /// type name
    pub type_name: Option<String>,
    /// type description
    pub type_description: Option<String>,
    /// year of vehicle (non-standard)
    pub year: Option<u32>,
}

impl ModDetailVehicleSorting {
    /// create new sorting sub-record
    fn new() -> Self {
        ModDetailVehicleSorting {
            brand: None,
            category: None,
            combos: vec![],
            name: None,
            type_name: None,
            type_description: None,
            year: None,
        }
    }
}

/// Vehicle Capability
pub enum VehicleCapability {
    /// Has option
    Yes,
    /// Does not have option
    No,
}

/// Vehicle flags
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleFlags {
    /// has beacon lights
    pub beacons: VehicleCapability,
    /// has paint options
    pub color: VehicleCapability,
    /// can be entered by player
    pub enterable: VehicleCapability,
    /// has real lights
    pub lights: VehicleCapability,
    /// is motorized
    pub motorized: VehicleCapability,
    /// has wheel options
    pub wheels: VehicleCapability,
}

impl ModDetailVehicleFlags {
    /// Create new vehicle flag sub-record
    fn new() -> Self {
        ModDetailVehicleFlags {
            beacons: VehicleCapability::No,
            color: VehicleCapability::No,
            enterable: VehicleCapability::No,
            lights: VehicleCapability::No,
            motorized: VehicleCapability::No,
            wheels: VehicleCapability::No,
        }
    }
}

impl Serialize for VehicleCapability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            VehicleCapability::Yes => serializer.serialize_bool(true),
            VehicleCapability::No => serializer.serialize_bool(false),
        }
    }
}

/// Vehicle engine sub-record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleEngine {
    /// fuel type
    pub fuel_type: Option<String>,
    /// transmission type (primary)
    pub transmission_type: Option<String>,
    /// motor configurations
    pub motors: Vec<MotorEntry>,
}

impl ModDetailVehicleEngine {
    /// create new engine sub-record
    fn new() -> Self {
        ModDetailVehicleEngine {
            fuel_type: None,
            transmission_type: None,
            motors: vec![],
        }
    }
}

/// Vehicle spray variant
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailSprayType {
    /// fill types supported
    pub fills: Vec<String>,
    /// working width
    pub width: Option<f32>,
}

/// Vehicle fill and spray sub-record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleFillSpray {
    /// fill categories for storage
    pub fill_cat: Vec<String>,
    /// capacity for storage
    pub fill_level: u32,
    /// fill types for storage
    pub fill_type: Vec<String>,
    /// list of spray variants
    pub spray_types: Vec<ModDetailSprayType>,
}

impl ModDetailVehicleFillSpray {
    /// create new fill and spray sub-record
    fn new() -> Self {
        ModDetailVehicleFillSpray {
            fill_cat: vec![],
            fill_level: 0,
            fill_type: vec![],
            spray_types: vec![],
        }
    }
}

/// Vehicle spec sub-record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicleSpecs {
    /// vehicle functions
    pub functions: Vec<String>,
    /// this vehicle can use tools that want to connect to these joints
    pub joint_accepts: Vec<String>,
    /// this vehicle needs to connect to these type of joints
    pub joint_requires: Vec<String>,
    /// vehicle name
    pub name: String,
    /// vehicle price
    pub price: u32,
    /// list of included specs
    pub specs: HashMap<String, u32>,
    /// vehicle weight
    pub weight: u32,
}

impl ModDetailVehicleSpecs {
    /// create new vehicle specs sub-record
    fn new() -> Self {
        ModDetailVehicleSpecs {
            functions: vec![],
            joint_accepts: vec![],
            joint_requires: vec![],
            name: String::new(),
            price: 0,
            specs: HashMap::new(),
            weight: 0,
        }
    }
}

/// Vehicle storeItem record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailVehicle {
    /// fills and sprays
    pub fill_spray: ModDetailVehicleFillSpray,
    /// feature flags
    pub flags: ModDetailVehicleFlags,
    /// path to base game icon
    pub icon_base: Option<String>,
    /// base64 webp icon, if loaded
    pub icon_file: Option<String>,
    /// original icon path
    pub icon_orig: Option<String>,
    /// master type (vehicle)
    pub master_type: String,
    /// motor information
    pub motor: ModDetailVehicleEngine,
    /// File is a sub of a different item
    pub parent_item : Option<String>,
    /// sorting information
    pub sorting: ModDetailVehicleSorting,
    /// vehicle specs
    pub specs: ModDetailVehicleSpecs,
}

impl ModDetailVehicle {
    #[must_use]
    /// Create new vehicle record
    pub fn new() -> Self {
        ModDetailVehicle {
            fill_spray: ModDetailVehicleFillSpray::new(),
            flags: ModDetailVehicleFlags::new(),
            icon_base: None,
            icon_file: None,
            icon_orig: None,
            master_type: String::from("vehicle"),
            parent_item : None,
            motor: ModDetailVehicleEngine::new(),
            sorting: ModDetailVehicleSorting::new(),
            specs: ModDetailVehicleSpecs::new(),
        }
    }
}

impl Default for ModDetailVehicle {
    fn default() -> Self {
        Self::new()
    }
}

/// motor value definition (hp, kph, or mph)
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorValue {
    /// RPM value
    pub rpm: u32,
    /// other value
    pub value: u32,
}
impl MotorValue {
    #[must_use]
    /// Create new motor value with round numbers
    pub fn new(rpm: f32, value: f32) -> Self {
        MotorValue {
            rpm: Self::round_to_u32(rpm),
            value: Self::round_to_u32(value),
        }
    }
    /// Round input number and cast to `u32`
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn round_to_u32(num: f32) -> u32 {
        num.round() as u32
    }
}

/// motor definition
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorEntry {
    /// name of motor
    pub name: String,
    /// list of rpm->hp values
    pub horse_power: Vec<MotorValue>,
    /// maximum stated speed (from author)
    pub max_speed: u32,
    /// list of rpm->kph values
    pub speed_kph: Vec<MotorValue>,
    /// list of rpm->mph values
    pub speed_mph: Vec<MotorValue>,
}

impl MotorEntry {
    #[must_use]
    /// create new motor definition
    pub fn new(name: String, max_speed: u32) -> Self {
        MotorEntry {
            name,
            horse_power: vec![],
            max_speed,
            speed_kph: vec![],
            speed_mph: vec![],
        }
    }
}

/// placable sorting information sub-record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlaceSorting {
    /// category
    pub category: Option<String>,
    /// functions
    pub functions: Vec<String>,
    /// has color choices
    pub has_color: VehicleCapability,
    /// income generated per hour
    pub income_per_hour: u32,
    /// name of placeable
    pub name: Option<String>,
    /// price
    pub price: u32,
    /// type name
    pub type_name: Option<String>,
}

impl ModDetailPlaceSorting {
    /// create new placeable sorting sub-record
    fn new() -> Self {
        ModDetailPlaceSorting {
            category: None,
            functions: vec![],
            has_color: VehicleCapability::No,
            income_per_hour: 0,
            name: None,
            price: 0,
            type_name: None,
        }
    }
}

/// placable husbandry sub-record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlaceAnimals {
    /// is a beehive
    pub beehive_exists: bool,
    /// honey per day in liters
    pub beehive_per_day: u32,
    /// working radius in meters
    pub beehive_radius: u32,
    /// number of animals
    pub husbandry_animals: u32,
    /// is a husbandry
    pub husbandry_exists: bool,
    /// type of husbandry
    pub husbandry_type: Option<String>,
}

impl ModDetailPlaceAnimals {
    /// create new placeable husbandry sub-record
    fn new() -> Self {
        ModDetailPlaceAnimals {
            beehive_exists: false,
            beehive_per_day: 0,
            beehive_radius: 0,
            husbandry_animals: 0,
            husbandry_exists: false,
            husbandry_type: None,
        }
    }
}

/// placable storage sub-record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlaceStorage {
    /// number of objects for object storage types
    pub objects: Option<u32>,
    /// silo capacity
    pub silo_capacity: u32,
    /// is a silo?
    pub silo_exists: bool,
    /// silo fill categories
    pub silo_fill_cats: Vec<String>,
    /// silo fill types
    pub silo_fill_types: Vec<String>,
}

impl ModDetailPlaceStorage {
    /// create new placeable storage sub-record
    fn new() -> Self {
        ModDetailPlaceStorage {
            objects: None,
            silo_capacity: 0,
            silo_exists: false,
            silo_fill_cats: vec![],
            silo_fill_types: vec![],
        }
    }
}

/// Production ingredient list
pub type ProductionIngredients = Vec<ProductionIngredient>;
/// Production recipe (list of list of ingredients - ingredients in nested level are "OR", ingredient list in top level is "AND")
pub type ProductionRecipe = Vec<ProductionIngredients>;

/// Production ingredient
#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductionIngredient {
    /// quantity for ingredient
    pub amount: f32,
    /// fill type of ingredient
    pub fill_type: String,
}
impl ProductionIngredient {
    /// create new production ingredient
    #[must_use]
    pub fn new(fill_type: String, amount: f32) -> Self {
        ProductionIngredient { amount, fill_type }
    }
}

/// production boost type
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionBoost {
    /// quantity for boots
    amount: f32,
    /// amount of boost (0-1) percentage
    boost_factor: f32,
    /// fill type for boost
    fill_type: String,
}
impl ProductionBoost {
    /// create new boost type
    #[must_use]
    pub fn new(fill_type: String, amount: f32, boost_factor: f32) -> Self {
        ProductionBoost {
            amount,
            boost_factor,
            fill_type,
        }
    }
}

/// Placeable production record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailProduction {
    /// list of boosts
    pub boosts: Vec<ProductionBoost>,
    /// cost per hour
    pub cost_per_hour: f32,
    /// cycles per hour
    pub cycles_per_hour: f32,
    /// name of production
    pub name: String,
    /// output types - multiples are AND
    pub output: Vec<ProductionIngredient>,
    /// name parameters (if used)
    pub params: String,
    /// production recipe - items on root level are AND, items on second level are OR
    pub recipe: ProductionRecipe,
}

impl ModDetailProduction {
    /// create new placeable production record
    #[must_use]
    pub fn new() -> Self {
        ModDetailProduction {
            boosts: vec![],
            cost_per_hour: 1_f32,
            cycles_per_hour: 1_f32,
            name: String::from("--"),
            output: vec![],
            params: String::new(),
            recipe: vec![],
        }
    }
}

impl Default for ModDetailProduction {
    fn default() -> Self {
        Self::new()
    }
}

/// Placable record
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDetailPlace {
    /// beehive and husbandry
    pub animals: ModDetailPlaceAnimals,
    /// path to base game icon
    pub icon_base: Option<String>,
    /// base64 webp icon, if loaded
    pub icon_file: Option<String>,
    /// original icon path
    pub icon_orig: Option<String>,
    /// master type, is "placeable"
    pub master_type: String,
    /// File is a sub of a different item
    pub parent_item : Option<String>,
    /// production list
    pub productions: Vec<ModDetailProduction>,
    /// placeable sorting information
    pub sorting: ModDetailPlaceSorting,
    /// silos and object storage
    pub storage: ModDetailPlaceStorage,
}

impl ModDetailPlace {
    #[must_use]
    /// create new Placable record
    pub fn new() -> Self {
        ModDetailPlace {
            animals: ModDetailPlaceAnimals::new(),
            icon_base: None,
            icon_file: None,
            icon_orig: None,
            master_type: String::from("placeable"),
            parent_item : None,
            productions: vec![],
            sorting: ModDetailPlaceSorting::new(),
            storage: ModDetailPlaceStorage::new(),
        }
    }
}

impl Default for ModDetailPlace {
    fn default() -> Self {
        Self::new()
    }
}
