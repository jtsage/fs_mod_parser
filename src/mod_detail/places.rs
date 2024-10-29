//! Parse placeables and productions
use super::{xml_extract_text_as_opt_string, xml_extract_text_as_opt_u32};
use crate::mod_detail::structs::{
    ModDetailPlace, ModDetailProduction, ProductionBoost, ProductionIngredient,
    ProductionIngredients, VehicleCapability,
};
use crate::shared::files::AbstractFileHandle;
use crate::shared::{convert_mod_icon, extract_and_normalize_image};
use crate::ModParserOptions;
use std::collections::HashMap;

/// Parse a placeable
///
/// Also processed productions if found
///
/// # Sample output:
/// ```json
/// {
///    "animals": {
///        "beehiveExists": false,
///        "beehivePerDay": 0,
///        "beehiveRadius": 0,
///        "husbandryAnimals": 0,
///        "husbandryExists": false,
///        "husbandryType": null
///    },
///    "iconBase": null,
///    "iconFile": null,
///    "masterType": "placeable",
///    "productions": [
///    {
///        "boosts": [],
///        "costPerHour": 2,
///        "cyclesPerHour": 1,
///        "name": "$l10n_fillType_forage_mixing",
///        "output": [
///             { "amount": 2000.0, "fillType": "forage" }
///        ],
///        "params": "",
///        "recipe": [
///             [ { "amount": 1000.0, "fillType": "silage" } ],
///             [ { "amount": 500.0, "fillType": "drygrass_windrow" } ],
///             [ { "amount": 500.0, "fillType": "straw" } ]
///        ]
///    }
///    ],
///    "sorting": {
///        "category": "fences",
///        "functions": [ "$l10n_function_decoration" ],
///        "hasColor": false,
///        "incomePerHour": 0,
///        "name": "$l10n_shopItem_guardRailLevel01",
///        "price": 100,
///        "typeName": "placeable"
///    },
///    "storage": {
///        "objects": null,
///        "siloCapacity": 0,
///        "siloExists": false,
///        "siloFillCats": [],
///        "siloFillTypes": []
///    }
/// },
/// ```
pub fn place_parse(
    xml_tree: &roxmltree::Document,
    file_handle: &mut Box<dyn AbstractFileHandle>,
    options: &ModParserOptions,
) -> ModDetailPlace {
    let mut this_place = ModDetailPlace::default();

    place_parse_sorting(xml_tree, &mut this_place);
    place_parse_storage(xml_tree, &mut this_place);
    place_parse_animals(xml_tree, &mut this_place);

    for production in xml_tree
        .descendants()
        .filter(|n| n.has_tag_name("production"))
    {
        this_place
            .productions
            .push(place_parse_production(&production));
    }

    if !options.skip_detail_icons {
        let image_entry = extract_and_normalize_image(xml_tree, "image");

        image_entry.original.clone_into(&mut this_place.icon_orig);

        if let Some(filename) = image_entry.base_game {
            this_place.icon_base = Some(filename);
        } else if let Some(filename) = image_entry.local_file {
            if let Ok(file_content) = file_handle.as_bin(&filename) {
                this_place.icon_file = convert_mod_icon(file_content);
            }
        }
    }

    this_place
}

/// Parse productions
fn place_parse_production(xml_node: &roxmltree::Node) -> ModDetailProduction {
    let mut this_production = ModDetailProduction::default();

    // let single_input:ProductionRecipe = vec![];
    let mut mix_inputs: HashMap<String, ProductionIngredients> = HashMap::new();

    if let Some(name) = xml_node.attribute("name") {
        name.clone_into(&mut this_production.name);
    }
    if let Some(params) = xml_node.attribute("params") {
        params.clone_into(&mut this_production.params);
    }

    if let Some(costs) = xml_node.attribute("costsPerActiveHour") {
        if let Ok(value) = costs.parse::<f32>() {
            this_production.cost_per_hour = value;
        }
    } else if let Some(costs) = xml_node.attribute("costsPerActiveMinute") {
        if let Ok(value) = costs.parse::<f32>() {
            this_production.cost_per_hour = value * 60_f32;
        }
    } else if let Some(costs) = xml_node.attribute("costsPerActiveMonth") {
        if let Ok(value) = costs.parse::<f32>() {
            this_production.cost_per_hour = value / 24_f32;
        }
    }

    if let Some(cycles) = xml_node.attribute("cyclesPerHour") {
        if let Ok(value) = cycles.parse::<f32>() {
            this_production.cycles_per_hour = value;
        }
    } else if let Some(cycles) = xml_node.attribute("cyclesPerMinute") {
        if let Ok(value) = cycles.parse::<f32>() {
            this_production.cycles_per_hour = value * 60_f32;
        }
    } else if let Some(cycles) = xml_node.attribute("cyclesPerMonth") {
        if let Ok(value) = cycles.parse::<f32>() {
            this_production.cycles_per_hour = value / 24_f32;
        }
    }

    for output in xml_node.descendants().filter(|n| n.has_tag_name("output")) {
        // Vec<ProductionIngredient> (all are produced)
        let Some(fill) = output.attribute("fillType") else {
            continue;
        };
        let Some(amount) = output
            .attribute("amount")
            .and_then(|n| n.parse::<f32>().ok())
        else {
            continue;
        };

        this_production.output.push(ProductionIngredient::new(
            fill.to_owned().to_lowercase(),
            amount,
        ));
    }

    for input in xml_node.descendants().filter(|n| n.has_tag_name("input")) {
        // fillType and amount always appear for well-formed inputs
        let Some(fill) = input.attribute("fillType") else {
            continue;
        };
        let Some(amount) = input
            .attribute("amount")
            .and_then(|n| n.parse::<f32>().ok())
        else {
            continue;
        };

        match input.attribute("mix") {
            Some("boost") => {
                // a booster
                let Some(boost_factor) = input
                    .attribute("boostfactor")
                    .and_then(|n| n.parse::<f32>().ok())
                else {
                    continue;
                };
                this_production.boosts.push(ProductionBoost::new(
                    fill.to_owned().to_lowercase(),
                    amount,
                    boost_factor,
                ));
            }
            Some(index) => {
                //multi-input
                let this_mix = mix_inputs.entry(index.to_owned()).or_default();
                this_mix.push(ProductionIngredient::new(
                    fill.to_owned().to_lowercase(),
                    amount,
                ));
            }
            _ => {
                // single
                let this_input: ProductionIngredients = vec![ProductionIngredient::new(
                    fill.to_owned().to_lowercase(),
                    amount,
                )];
                this_production.recipe.push(this_input);
            }
        }
    }

    for mut multi_value in mix_inputs.into_values() {
        sort_by_key_prod_ingredient(&mut multi_value, |d| &d.fill_type);
        this_production.recipe.push(multi_value);
        // multi_value.sort_by_key("fill_type")
    }
    // this_production.recipe.extend(mix_inputs.into_values());

    this_production
}

/// sort production recipe ingredients by ingredient name
fn sort_by_key_prod_ingredient<T, F, K>(slice: &mut [T], f: F)
where
    F: for<'a> Fn(&'a T) -> &'a K,
    K: Ord,
{
    slice.sort_by(|a, b| f(a).cmp(f(b)));
}

/// Parse storage (bales and silos)
fn place_parse_storage(xml_tree: &roxmltree::Document, this_place: &mut ModDetailPlace) {
    if let Some(obj_store) = xml_tree
        .root()
        .descendants()
        .find(|n| n.has_tag_name("objectStorage"))
    {
        this_place.storage.objects =
            Some(str::parse(obj_store.attribute("capacity").unwrap_or("250")).unwrap_or(250));
    }

    if let Some(silo_node) = xml_tree
        .descendants()
        .find(|n| n.has_tag_name("silo") || n.has_tag_name("siloExtension"))
    {
        let mut capacity: Vec<Option<&str>> = vec![];

        for fill_unit in silo_node
            .descendants()
            .filter(|n| n.has_tag_name("storage"))
        {
            capacity.push(fill_unit.attribute("capacity"));

            if let Some(cats) = fill_unit.attribute("fillTypeCategories") {
                this_place
                    .storage
                    .silo_fill_cats
                    .extend(cats.split(' ').map(|n| n.to_lowercase().clone()));
            }
            if let Some(cats) = fill_unit.attribute("fillTypes") {
                this_place
                    .storage
                    .silo_fill_types
                    .extend(cats.split(' ').map(|n| n.to_lowercase().clone()));
            }
        }

        this_place.storage.silo_capacity = capacity
            .into_iter()
            .flatten()
            .flat_map(str::parse::<u32>)
            .sum();

        if this_place.storage.silo_capacity > 0 {
            this_place.storage.silo_exists = true;
        }

        this_place.storage.silo_fill_cats.sort();
        this_place.storage.silo_fill_cats.dedup();
        this_place.storage.silo_fill_types.sort();
        this_place.storage.silo_fill_types.dedup();
    }
}

/// Parse animal husbandry and beehives
fn place_parse_animals(xml_tree: &roxmltree::Document, this_place: &mut ModDetailPlace) {
    if let Some(this_beehive) = xml_tree.descendants().find(|n| n.has_tag_name("beehive")) {
        this_place.animals.beehive_exists = true;
        this_place.animals.beehive_per_day =
            str::parse(this_beehive.attribute("litersHoneyPerDay").unwrap_or("0")).unwrap_or(0);
        this_place.animals.beehive_radius =
            str::parse(this_beehive.attribute("actionRadius").unwrap_or("0")).unwrap_or(0);
    }

    if let Some(this_pen) = xml_tree.descendants().find(|n| n.has_tag_name("animals")) {
        this_place.animals.husbandry_exists = true;
        this_place.animals.husbandry_animals =
            str::parse(this_pen.attribute("maxNumAnimals").unwrap_or("0")).unwrap_or(0);
        this_place.animals.husbandry_type = this_pen
            .attribute("type")
            .map(std::string::ToString::to_string);
    }
}

/// Parse placeable sorting data
fn place_parse_sorting(xml_tree: &roxmltree::Document, this_place: &mut ModDetailPlace) {
    this_place.sorting.category = xml_extract_text_as_opt_string(xml_tree, "category");
    this_place.sorting.income_per_hour =
        xml_extract_text_as_opt_u32(xml_tree, "incomePerHour").unwrap_or(0);
    this_place.sorting.name = xml_extract_text_as_opt_string(xml_tree, "name");
    this_place.sorting.price = xml_extract_text_as_opt_u32(xml_tree, "price").unwrap_or(0);
    this_place.sorting.type_name = xml_tree
        .root_element()
        .attribute("type")
        .map(std::string::ToString::to_string);

    if xml_tree
        .descendants()
        .filter(|n| n.has_tag_name("color"))
        .count()
        > 1
    {
        this_place.sorting.has_color = VehicleCapability::Yes;
    }

    this_place.sorting.functions = xml_tree
        .descendants()
        .filter(|n| n.has_tag_name("function"))
        .filter_map(|n| n.text())
        .map(std::string::ToString::to_string)
        .collect();
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shared::files::AbstractNull;
    use assert_json_diff::assert_json_include;
    use serde_json::json;

    #[test]
    fn base_game_icon() {
        /* cSpell: disable */
        let minimum_xml = r#"<vehicle><storeData>
            <image>$data/vehicles/albutt/frontloaderShovel/store_albuttFrontloaderShovel.png</image>
            </storeData></vehicle>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();

        let mut file_handle: Box<dyn AbstractFileHandle> = Box::new(AbstractNull::new().unwrap());
        let this_place = place_parse(&minimum_doc, &mut file_handle, &ModParserOptions::default());

        assert_eq!(
            this_place.icon_base,
            Some(String::from(
                "$data/vehicles/albutt/frontloaderShovel/store_albuttFrontloaderShovel.png"
            ))
        );
        assert_eq!(this_place.icon_file, None);
        /* cSpell: enable */
    }

    #[test]
    fn placeable_animal_beehive() {
        let minimum_xml = r#"<placeable>
            <beehive actionRadius="25" litersHoneyPerDay="5"></beehive>
            </placeable>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let mut this_place = ModDetailPlace::default();

        place_parse_animals(&minimum_doc, &mut this_place);

        let actual = json!(this_place.animals);
        let expected = json!({
            "beehiveExists": true,
            "beehivePerDay": 5,
            "beehiveRadius": 25,
            "husbandryAnimals": 0,
            "husbandryExists": false,
            "husbandryType": null
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
    }

    #[test]
    fn placeable_animal_cows() {
        let minimum_xml = r#"<placeable>
            <husbandry saveId="Animals_COW" hasStatistics="false">
                <animals type="COW" maxNumAnimals="15" ></animals>
            </husbandry>
            </placeable>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let mut this_place = ModDetailPlace::default();

        place_parse_animals(&minimum_doc, &mut this_place);

        let actual = json!(this_place.animals);
        let expected = json!({
            "beehiveExists": false,
            "beehivePerDay": 0,
            "beehiveRadius": 0,
            "husbandryAnimals": 15,
            "husbandryExists": true,
            "husbandryType": "COW"
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
    }

    #[test]
    fn placeable_silo_extension() {
        /* cSpell: disable */
        let minimum_xml = r#"<placeable>
            <siloExtension nearSiloWarning="$l10n_warning_liquidManureTankNotNearBarn">
                <storage node="storage" fillTypes="LIQUIDMANURE" capacity="2600000" isExtension="true" >
                </storage>
            </siloExtension>
            </placeable>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let mut this_place = ModDetailPlace::default();

        place_parse_storage(&minimum_doc, &mut this_place);

        let actual = json!(this_place.storage);
        let expected = json!({
            "objects": null,
            "siloCapacity": 2600000,
            "siloExists": true,
            "siloFillCats": [],
            "siloFillTypes": [ "liquidmanure" ]
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
        /* cSpell: enable */
    }

    #[test]
    fn placeable_silo() {
        /* cSpell: disable */
        let minimum_xml = r#"<placeable>
            <silo>
                <storages perFarm="true">
                    <storage fillTypeCategories="TRAINWAGON" capacity="500000" costsPerFillLevelAndDay="0.005" isExtension="false"/>
                </storages>
            </silo>
            </placeable>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let mut this_place = ModDetailPlace::default();

        place_parse_storage(&minimum_doc, &mut this_place);

        let actual = json!(this_place.storage);
        let expected = json!({
            "objects": null,
            "siloCapacity": 500000,
            "siloExists": true,
            "siloFillCats": [ "trainwagon"],
            "siloFillTypes": []
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
        /* cSpell: enable */
    }

    #[test]
    fn placeable_object_storage() {
        /* cSpell: disable */
        let minimum_xml = r#"<placeable>
            <objectStorage supportsBales="true" supportsPallets="true" maxLength="8.5" maxWidth="6" maxHeight="3.5">
            </objectStorage>
            </placeable>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let mut this_place = ModDetailPlace::default();

        place_parse_storage(&minimum_doc, &mut this_place);

        let actual = json!(this_place.storage);
        let expected = json!({
            "objects": 250,
            "siloCapacity": 0,
            "siloExists": false,
            "siloFillCats": [],
            "siloFillTypes": []
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
        /* cSpell: enable */
    }

    #[test]
    fn placeable_production_params() {
        /* cSpell: disable */
        let minimum_xml = r#"<production id="fabric_cotton" name="%s %s" params="$l10n_fillType_fabric|$l10n_fillType_cotton" cyclesPerHour="24" costsPerActiveHour="5">
                <inputs>
                    <input fillType="COTTON" amount="5" />
                </inputs>
                <outputs>
                    <output fillType="FABRIC" amount="3" />
                </outputs>
            </production>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let this_place = place_parse_production(&minimum_doc.root_element());

        let actual = json!(this_place);
        let expected = json!({
            "boosts": [],
            "costPerHour": 5.0,
            "cyclesPerHour": 24.0,
            "name": "%s %s",
            "output": [ { "amount": 3.0, "fillType": "fabric" } ],
            "params": "$l10n_fillType_fabric|$l10n_fillType_cotton",
            "recipe": [
                [ { "amount": 5.0, "fillType": "cotton" } ]
            ]
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
        /* cSpell: enable */
    }

    #[test]
    fn placeable_duration_in_months() {
        /* cSpell: disable */
        let minimum_xml = r#"<production id="fabric_cotton" name="%s %s" params="$l10n_fillType_fabric|$l10n_fillType_cotton" cyclesPerMonth="240" costsPerActiveMonth="300">
                <inputs>
                    <input fillType="COTTON" amount="5" />
                </inputs>
                <outputs>
                    <output fillType="FABRIC" amount="3" />
                </outputs>
            </production>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let this_place = place_parse_production(&minimum_doc.root_element());

        let actual = json!(this_place);
        let expected = json!({
            "boosts": [],
            "costPerHour": 12.5,
            "cyclesPerHour": 10.0,
            "name": "%s %s",
            "output": [ { "amount": 3.0, "fillType": "fabric" } ],
            "params": "$l10n_fillType_fabric|$l10n_fillType_cotton",
            "recipe": [
                [ { "amount": 5.0, "fillType": "cotton" } ]
            ]
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
        /* cSpell: enable */
    }

    #[test]
    fn placeable_duration_in_minutes() {
        /* cSpell: disable */
        let minimum_xml = r#"<production id="fabric_cotton" name="%s %s" params="$l10n_fillType_fabric|$l10n_fillType_cotton" cyclesPerMinute="4" costsPerActiveMinute="3">
                <inputs>
                    <input fillType="COTTON" amount="5" />
                </inputs>
                <outputs>
                    <output fillType="FABRIC" amount="3" />
                </outputs>
            </production>"#;
        let minimum_doc = roxmltree::Document::parse(&minimum_xml).unwrap();
        let this_place = place_parse_production(&minimum_doc.root_element());

        let actual = json!(this_place);
        let expected = json!({
            "boosts": [],
            "costPerHour": 180.0,
            "cyclesPerHour": 240.0,
            "name": "%s %s",
            "output": [ { "amount": 3.0, "fillType": "fabric" } ],
            "params": "$l10n_fillType_fabric|$l10n_fillType_cotton",
            "recipe": [
                [ { "amount": 5.0, "fillType": "cotton" } ]
            ]
        });
        // assert_eq!(actual.to_string(), expected.to_string());
        assert_json_include!(actual : actual, expected : expected);
        /* cSpell: enable */
    }
}
