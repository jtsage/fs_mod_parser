use std::collections::HashMap;

use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth, PathType};
use crate::files::store_item::Capability;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

// MARK: Placeable
/// Vehicle storeItem record
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Placeable {
    /// path to base game icon
    pub icon_base: Option<String>,
    /// path to local icon
    pub icon_file: Option<String>,
    /// icon data
    pub icon_data: Option<String>,
    /// beehive and husbandry
    pub animals: Animals,
    /// File is a sub of a different item
    pub parent_item: Option<String>,
    /// production list
    pub productions: Vec<Production>,
    /// show in store
    pub show_in_store: bool,
    /// placeable sorting information
    pub sorting: Sorting,
    /// silos and object storage
    pub storage: Vec<Storage>,
}

impl Default for Placeable {
    fn default() -> Self {
        Self { icon_base: None, icon_file: None, icon_data: None, animals: Animals::default(), parent_item: None, productions: Vec::default(), show_in_store: true, sorting: Sorting::default(), storage: Vec::default() }
    }
}

// MARK: Sorting
/// Vehicle sorting data
#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Sorting {
    /// category
    pub category: Option<String>,
    /// functions
    pub functions: Vec<String>,
    /// has color choices
    pub has_color: Capability,
    /// Has lighting
    pub has_lights: Capability,
    /// income generated per hour
    pub income_per_hour: u32,
    /// name of placeable
    pub name: Option<String>,
    /// price
    pub price: Option<u32>,
    /// type name
    pub type_name: Option<String>,
    /// show in store
    pub show_in_store : bool,
}

// MARK: Animals
/// Husbandry data
#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Animals {
    /// honey per day in liters
    pub beehive_per_day: Option<u32>,
    /// working radius in meters
    pub beehive_radius: Option<u32>,
    /// number of animals
    pub husbandry_count: Option<u32>,
    /// type of husbandry
    pub husbandry_type: Option<String>,
    /// food capacity
    pub husbandry_food: Option<u32>,
}

// MARK: Storage
/// Storage - objects or fill
#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    /// This is an object storage
    pub is_object: Option<ObjectStorage>,
    /// capacity
    pub capacity: u32,
    /// fill categories
    pub categories: Vec<String>,
    /// fill types
    pub types: Vec<String>,
}

/// Object storage types
#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObjectStorage {
    /// Bales
    Bales,
    /// Pallets
    Pallets,
    /// Bales and Pallets
    Both,
}


// MARK: XMLReader
impl XMLReader<Self> for Placeable {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
    }

    #[inline]
    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"placeable", 0) => {
                self.sorting.type_name = Self::xml_attribute(e, "type");
                Ok(1)
            },
            (_, 0) => Err(AbstractFileError::XmlWrongFileType),

            (b"image", 2) => {
                if let Some(filename) = Self::xml_text(e, reader) {
                    let filename = Self::unwrap_base_path(filename);
                    match &filename {
                        PathType::Base(_) => self.icon_base = Some(filename.clone().to_dds()),
                        PathType::Local(_) => self.icon_file = Some(filename.clone().to_dds()),
                    }
                }
                Ok(0)
            },
            (b"parentFile", 1) => {
                self.parent_item = Self::xml_attribute(e, "xmlFilename");
                Self::slurp(e, reader)
            },
            (b"showInStore", 2) => {
                if let Some(v) = Self::xml_text(e, reader) {
                    if v == *"false" {
                        self.show_in_store = false;
                    }
                }
                Ok(0)
            },

            // MARK: ~sorting
            (b"category", 2) => { self.sorting.category = Self::xml_text(e, reader); Ok(0) },
            (b"name", 2) => { self.sorting.name = Self::xml_text(e, reader); Ok(0) },
            (b"function", 3) => { 
                if let Some(v) = Self::xml_text(e, reader) { self.sorting.functions.push(v) }
                Ok(0)
            },
            (b"price", 2) => { self.sorting.price = Self::xml_number(e, reader); Ok(0) },
            (b"incomePerHour", 1) => { 
                if let Some(income) = Self::xml_number(e, reader) {
                    self.sorting.income_per_hour = std::cmp::max(self.sorting.income_per_hour, income);
                }
                Ok(0)
            },
            (b"solarPanelsConfiguration", _) => {
                if let Some(income) = Self::xml_attribute_number(e, "incomePerHour") {
                    self.sorting.income_per_hour = std::cmp::max(self.sorting.income_per_hour, income);
                }
                Ok(1)
            },
            (b"colorConfigurations" | b"colorable", 1) => {
                self.sorting.has_color = Capability::Yes; Self::slurp(e, reader)
            },
            (b"realLights", 2) => {
                self.sorting.has_lights = Capability::Yes; Self::slurp(e, reader)
            },

            // MARK: ~animals
            (b"husbandry", 1) => { self.tag_husbandry(reader, e); Ok(0) },
            (b"beehive", 1) => {
                self.animals.beehive_per_day = Self::xml_attribute_number(e, "litersHoneyPerDay");
                self.animals.beehive_radius = Self::xml_attribute_number(e, "actionRadius");
                Self::slurp(e, reader)
            },
            (b"siloExtension", 1) | (b"storages", 2) => {
                self.tag_silo(reader, e);
                Ok(0)
            }
            (b"productionPoint", 1) => { self.tag_productions(reader, e); Ok(0) },
            (b"objectStorage", 1) => {
                let pallets = Self::xml_attribute(e, "supportsPallets").is_some_and(|v| v == *"true");
                let bales = Self::xml_attribute(e, "supportsBales").is_some_and(|v| v == *"true");
                let capacity = Self::xml_attribute_number(e, "capacity").unwrap_or(250_u32);

                self.storage.push(Storage{
                    is_object: match (pallets, bales) {
                        (true, true) => Some(ObjectStorage::Both),
                        (true, false) => Some(ObjectStorage::Pallets),
                        (false, true) => Some(ObjectStorage::Bales),
                        _ => None
                    },
                    capacity,
                    categories: vec![],
                    types: vec![],
                });
                Self::slurp(e, reader)
            }
            _ => Ok(1),
        }
    }

    fn tags_self_closing(&mut self, e: &BytesStart, _depth : i32) {
        if e.name().as_ref() == b"solarPanelsConfiguration" {
            if let Some(income) = Self::xml_attribute_number(e, "incomePerHour") {
                self.sorting.income_per_hour = std::cmp::max(self.sorting.income_per_hour, income);
            }
        }
    }
}

impl Placeable {
    // MARK: __storage
    /// Do a storage line
    fn tag_storage(&mut self, e: &BytesStart) {
        let mut storage = Storage::default();

        if let Some(v) = Self::xml_attribute_number(e, "capacity") {
            storage.capacity = v;
        }
        if let Some(v) = Self::xml_attribute(e, "fillType") {
            storage.types = v.split_whitespace().filter_map(|v| if v == "unknown" { None } else { Some(v.to_ascii_lowercase()) }).collect();
        }
        if let Some(v) = Self::xml_attribute(e, "fillTypes") {
            storage.types = v.split_whitespace().filter_map(|v| if v == "unknown" { None } else { Some(v.to_ascii_lowercase()) }).collect();
        }
        if let Some(v) = Self::xml_attribute(e, "fillTypeCategories") {
            storage.categories = v.split_whitespace().map(str::to_ascii_lowercase).collect();
        }
        if storage.capacity != 0 {
            self.storage.push(storage);
        }
    }

    // MARK: _silo
    /// Do silos and extensions
    #[inline]
    fn tag_silo(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) {
        let mut buf_next = Vec::new();
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"storage" => {
                    self.tag_storage(&e);
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }
    }

    // MARK: _husbandry
    /// Do husbandry
    #[inline]
    fn tag_husbandry(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) {
        let mut buf_next = Vec::new();
        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e) | Event::Empty(e)) => {
                    match e.name().as_ref() {
                        b"animals" => {
                            self.animals.husbandry_type = Self::xml_attribute(&e, "type");
                            self.animals.husbandry_count = Self::xml_attribute_number(&e, "maxNumAnimals");
                        },
                        b"food" => {
                            self.animals.husbandry_food = Self::xml_attribute_number(&e, "capacity");
                        },
                        b"capacity" => {
                            self.tag_storage(&e);
                        },
                        _ => ()
                    }
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }
    }

    // MARK: _productions
    /// Do parent productions point tag
    #[inline]
    fn tag_productions(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) {
        let mut buf_next = Vec::new();
        loop {
                
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"capacity" => {
                    self.tag_storage(&e);
                },
                Ok(Event::Start(e)) if e.name().as_ref() == b"production" => {
                    self.tag_production(reader, &e);
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }
    }

    // MARK: _productions
    /// Do production
    #[inline]
    fn tag_production(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) {
        let mut buf_next = Vec::new();
        let mut production = Production::default();

        if let Some(name) = Self::xml_attribute(e, "name") {
            name.clone_into(&mut production.name);
        }
        if let Some(params) = Self::xml_attribute(e, "params") {
            let params = params.split('|').map(std::string::ToString::to_string).collect();
            production.params = Some(params);
        }

        if let Some(costs) = Self::xml_attribute_number::<f32>(e, "costsPerActiveHour") {
            production.cost_per_hour = costs;
        } else if let Some(costs) = Self::xml_attribute_number::<f32>(e, "costsPerActiveMinute") {
            production.cost_per_hour = costs * 60_f32;
        } else if let Some(costs) = Self::xml_attribute_number::<f32>(e, "costsPerActiveMonth") {
            production.cost_per_hour = costs / 24_f32;
        }
    
        if let Some(cycles) = Self::xml_attribute_number::<f32>(e, "cyclesPerHour") {
            production.cycles_per_hour = cycles;
        } else if let Some(cycles) = Self::xml_attribute_number::<f32>(e, "cyclesPerMinute") {
            production.cycles_per_hour = cycles * 60_f32;
        } else if let Some(cycles) = Self::xml_attribute_number::<f32>(e, "cyclesPerMonth") {
            production.cycles_per_hour = cycles / 24_f32;
        }

        let mut mix_map:HashMap<String, Vec<Ingredient>> = HashMap::new();

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e)) => {
                    match e.name().as_ref() {
                        b"output" => {
                            let Some(fill_type) = Self::xml_attribute(&e, "fillType").map(|v| v.to_ascii_lowercase()) else { continue };
                            let Some(quantity) = Self::xml_attribute_number(&e, "amount") else { continue };
                            production.output.push(Ingredient { quantity, fill_type, ..Default::default() });
                        },
                        b"input" => {
                            let Some(fill_type) = Self::xml_attribute(&e, "fillType").map(|v| v.to_ascii_lowercase()) else { continue };
                            let Some(quantity) = Self::xml_attribute_number(&e, "amount") else { continue };

                            match Self::xml_attribute(&e, "mix") {
                                None => {
                                    production.recipe.push(vec![Ingredient{ quantity, fill_type, ..Default::default()}]);
                                },
                                Some(v) if v == *"boost" => {
                                    let factor = Self::xml_attribute_number::<f32>(&e, "boostfactor").unwrap_or(0.01);
                                    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                                    let factor = Some((factor * 100.0).round() as u32);
                                    production.boosts.push(Ingredient{ quantity, factor, fill_type });
                                }
                                Some(v) => {
                                    let entry = mix_map.entry(v).or_default();
                                    entry.push(Ingredient { quantity, fill_type, ..Default::default() });
                                }
                            };
                        },
                        _ => ()
                    }
                },
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }

        // production.boosts.sort();
        // production.output.sort();

        let mut mixes:Vec<String> = mix_map.clone().into_keys().collect();
        mixes.sort_unstable();

        for key in mixes {
            if let Some(item) = mix_map.remove(&key) {
                production.recipe.push(item);
            }
        }

        self.productions.push(production);
    }
}




//MARK: PRODUCTION
/// Production ingredient list
pub type Ingredients = Vec<Ingredient>;
/// Production recipe (list of list of ingredients - ingredients in nested level are "OR", ingredient list in top level is "AND")
pub type Recipe = Vec<Ingredients>;

/// production ingredient type
#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Ingredient {
    /// quantity
    quantity: u32,
    /// amount of boost percentage
    factor: Option<u32>,
    /// fill type
    fill_type: String,
}

/// Placeable production record
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Production {
    /// list of boosts
    pub boosts: Ingredients,
    /// cost per hour
    pub cost_per_hour: f32,
    /// cycles per hour
    pub cycles_per_hour: f32,
    /// name of production
    pub name: String,
    /// output types - multiples are AND
    pub output: Ingredients,
    /// name parameters (if used)
    pub params: Option<Vec<String>>,
    /// production recipe - items on root level are AND, items on second level are OR
    pub recipe: Recipe,
}

impl Default for Production {
    fn default() -> Self {
        Self { boosts: Ingredients::default(), cost_per_hour: 1.0, cycles_per_hour: 1.0, name: String::from("--"), output: Ingredients::default(), params: None, recipe: Recipe::default() }
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
        <placeable>
            <storeData><image>$data/path/to/data/file.png</image></storeData>
        </placeable>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let placeable = actual.placeable.unwrap();

        assert_eq!(placeable.icon_base, Some(String::from("path/to/data/file.dds")));
        assert!(placeable.icon_file.is_none());

        let xml = xml_test(r#"
            <placeable>
                <storeData><image>local/path/to/data/file.png</image></storeData>
            </placeable>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let placeable = actual.placeable.unwrap();

        assert_eq!(placeable.icon_file, Some(String::from("local/path/to/data/file.dds")));
        assert!(placeable.icon_base.is_none());

        let xml = xml_test(r#"
            <placeable>
                <storeData><image></image></storeData>
            </placeable>"#);
        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let placeable = actual.placeable.unwrap();

        assert_eq!(placeable.icon_file, Some(String::from("")));
        assert!(placeable.icon_base.is_none());
    }


    #[test]
    fn show_in_store() {
        let xml = xml_test(r#"<placeable>
            <storeData><showInStore>true</showInStore></storeData>
        </placeable>"#);

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        assert_eq!(place.show_in_store, true);

        let xml = xml_test(r#"<placeable>
            <storeData><showInStore>false</showInStore></storeData>
        </placeable>"#);

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        assert_eq!(place.show_in_store, false);
    }

    #[test]
    fn beehive() {
        let xml = xml_test(r#"<placeable>
            <beehive actionRadius="50" litersHoneyPerDay="20"></beehive>
        </placeable>"#);

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        assert_eq!(place.animals.beehive_per_day, Some(20));
        assert_eq!(place.animals.beehive_radius, Some(50));
    }

    #[test]
    fn husbandry() {
        // cSpell:disable
        let xml = xml_test(r#"<placeable>
            <husbandry saveId="Animals_COW" hasStatistics="false">
                <animals type="COW" maxNumAnimals="200" maxNumVisualAnimals="50"></animals>
                <food capacity="150750"></food>
                <storage node="robotys" fillTypes="LIQUIDMANURE BUFFALOMILK MILK STRAW MANURE" isExtension="false">
                    <capacity fillType="LIQUIDMANURE" capacity="156250" />
                    <capacity fillType="MILK" capacity="133750" />
                    <capacity fillType="BUFFALOMILK" capacity="133750" />
                    <capacity fillType="STRAW" capacity="121375" />
                    <capacity fillType="MANURE" capacity="0" />
                </storage>
            </husbandry>
        </placeable>"#);
        

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        let expected = vec![
            Storage { is_object: None, capacity: 156_250, categories: vec![], types: vec![String::from("liquidmanure")] },
            Storage { is_object: None, capacity: 133_750, categories: vec![], types: vec![String::from("milk")] },
            Storage { is_object: None, capacity: 133_750, categories: vec![], types: vec![String::from("buffalomilk")] },
            Storage { is_object: None, capacity: 121_375, categories: vec![], types: vec![String::from("straw")] },
        ];
        // cSpell:enable

        assert_eq!(place.animals.husbandry_count, Some(200));
        assert_eq!(place.animals.husbandry_food, Some(150750));
        assert_eq!(place.animals.husbandry_type, Some(String::from("COW")));
        assert_eq!(place.storage, expected);
    }

    #[test]
    fn income_from_22() {
        let xml = xml_test(r#"<placeable>
            <incomePerHour>372</incomePerHour>
        </placeable>"#);

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        assert_eq!(place.sorting.income_per_hour, 372);
    }

    #[test]
    fn income_from_25() {
        let xml = xml_test(r#"<placeable>
            <solarPanels>
                <solarPanelsConfigurations>
                    <solarPanelsConfiguration name="$l10n_ui_no" isActive="false">
                        <objectChange node="solarPanels" visibilityActive="false"/>
                    </solarPanelsConfiguration>
                    <solarPanelsConfiguration name="$l10n_ui_yes" isActive="true" price="35000" incomePerHour="100"/>
                </solarPanelsConfigurations>
            </solarPanels>
        </placeable>"#);

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        assert_eq!(place.sorting.income_per_hour, 100);
    }

    #[test]
    fn parent_file() {
        let xml = xml_test(r#"<placeable>
            <parentFile xmlFilename="$data/path/to/file.xml"></parentFile>
        </placeable>"#);

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        assert_eq!(place.parent_item, Some(String::from("$data/path/to/file.xml")));
    }

    #[test]
    fn silo() {
        // cSpell:disable
        let xml = xml_test(r#"<placeable>
            <silo>
                <storages>
                    <storage node="storage" fillTypeCategories="farmSilo" capacity="980000" isExtension="false"/>
                </storages>
            </silo>
        </placeable>"#);
        // cSpell:enable

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        let expected = vec![
            // cSpell:disable-next-line
            Storage { is_object: None, capacity: 980_000, categories: vec![String::from("farmsilo")], types: vec![] },
        ];

        assert_eq!(place.storage, expected);
    }

    #[test]
    fn silo_extension() {
        // cSpell:disable
        let xml = xml_test(r#"<placeable>
            <siloExtension>
                <storage node="storage" fillTypeCategories="farmSilo" capacity="250000" isExtension="true"/>
            </siloExtension>
        </placeable>"#);
        // cSpell:enable

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        let expected = vec![
            // cSpell:disable-next-line
            Storage { is_object: None, capacity: 250_000, categories: vec![String::from("farmsilo")], types: vec![] },
        ];

        assert_eq!(place.storage, expected);
    }

    #[test]
    fn object_storage() {
        // cSpell:disable
        let xml = xml_test(r#"<placeable>
            <objectStorage supportsBales="true" supportsPallets="true" maxLength="8.5" maxWidth="6" maxHeight="3.5">
            </objectStorage>
        </placeable>"#);
        // cSpell:enable

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        let expected = vec![
            Storage { is_object: Some(ObjectStorage::Both), capacity: 250, categories: vec![], types: vec![] },
        ];

        assert_eq!(place.storage, expected);
    }


    #[test]
    fn production_silo() {
        // cSpell:disable
        let xml = xml_test(r#"<placeable>
            <productionPoint>
                <storage isExtension="false" fillLevelSyncThreshold="50">
                    <capacity fillType="POTATO"   capacity="1000000" />
                    <capacity fillType="SUGARBEET_CUT"   capacity="2000000" />
                    <capacity fillType="SUGARBEET"   capacity="1000000" />
                </storage>
            </productionPoint>
        </placeable>"#);
        // cSpell:enable

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        let expected = vec![
            Storage { is_object: None, capacity: 1_000_000, categories: vec![], types: vec![String::from("potato")] },
            Storage { is_object: None, capacity: 2_000_000, categories: vec![], types: vec![String::from("sugarbeet_cut")] },
            Storage { is_object: None, capacity: 1_000_000, categories: vec![], types: vec![String::from("sugarbeet")] },
        ];

        assert_eq!(place.storage, expected);
    }

    #[test]
    fn production_params() {
        // cSpell:disable
        let xml = xml_test(r#"<placeable>
            <productionPoint>
                <production id="fabric_cotton" name="%s %s" params="$l10n_fillType_fabric|$l10n_fillType_cotton" cyclesPerMinute="4" costsPerActiveMinute="3">
                <inputs><input fillType="COTTON" amount="5" /></inputs>
                <outputs><output fillType="FABRIC" amount="3" /></outputs>
            </production>
            </productionPoint>
        </placeable>"#);
        // cSpell:enable

        let actual = StoreItem::from_string(xml.as_ref()).unwrap();
        let place = actual.placeable.unwrap();

        assert_eq!(place.productions[0].params, Some(vec![String::from("$l10n_fillType_fabric"), String::from("$l10n_fillType_cotton")]));
        assert_eq!(place.productions[0].output, vec![Ingredient{ fill_type: String::from("fabric"), quantity : 3, factor: None}]);
        assert_eq!(place.productions[0].recipe[0], vec![Ingredient{ fill_type: String::from("cotton"), quantity : 5, factor: None}]);
    }


    #[test]
    fn from_file_husband() {
        let filename = "tests/test_mods/DETAIL_Samples.zip";
        let item = "xml/place-husbandry.xml";
        let json = "json/place-husbandry.json";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = StoreItem::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = file_handle.text(json).unwrap();
        let re_read:StoreItem = serde_json::from_str(&expected).unwrap();

        assert_eq!(actual, re_read);
    }

    #[test]
    fn from_file_prod_simple() {
        let filename = "tests/test_mods/DETAIL_Samples.zip";
        let item = "xml/production-simple.xml";
        let json = "json/production-simple.json";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = StoreItem::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = file_handle.text(json).unwrap();
        let re_read:StoreItem = serde_json::from_str(&expected).unwrap();

        assert_eq!(actual, re_read);
    }

    #[test]
    fn from_file_prod_deep() {
        let filename = "tests/test_mods/DETAIL_Samples.zip";
        let item = "xml/production-deep.xml";
        let json = "json/production-deep.json";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = StoreItem::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{}", serde_json::to_string_pretty(&actual).unwrap()); }

        let expected = file_handle.text(json).unwrap();
        let re_read:StoreItem = serde_json::from_str(&expected).unwrap();

        assert_eq!(actual, re_read);
    }
}
