//! Parse placeables and productions
use std::collections::HashMap;
use crate::ModParserOptions;
use crate::mod_detail::structs::{VehicleCapability, ModDetailPlace, ModDetailProduction, ProductionIngredient, ProductionIngredients, ProductionBoost};
use crate::shared::files::AbstractFileHandle;
use super::{xml_extract_text_as_opt_string, xml_extract_text_as_opt_u32};
use crate::shared::{extract_and_normalize_image, convert_mod_icon};

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
pub fn place_parse(xml_tree : &roxmltree::Document, file_handle: &mut Box<dyn AbstractFileHandle>,  options : &ModParserOptions ) -> ModDetailPlace {
    let mut this_place = ModDetailPlace::new();
    
    place_parse_sorting(xml_tree, &mut this_place);
    place_parse_storage(xml_tree, &mut this_place);
    place_parse_animals(xml_tree, &mut this_place);

    for production in xml_tree.descendants().filter(|n|n.has_tag_name("production")) {
        this_place.productions.push(place_parse_production(&production));
    }

    if !options.skip_detail_icons {
        let image_entry = extract_and_normalize_image(xml_tree, "image");

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
fn place_parse_production(xml_node : &roxmltree::Node) -> ModDetailProduction {
    let mut this_production = ModDetailProduction::new();

    // let single_input:ProductionRecipe = vec![];
    let mut mix_inputs:HashMap<String, ProductionIngredients> = HashMap::new();

    if let Some(name) = xml_node.attribute("name") {
        name.clone_into(& mut this_production.name);
    }
    if let Some(params) = xml_node.attribute("params") {
        params.clone_into(&mut this_production.params);
    }

    if let Some(costs) = xml_node.attribute("costsPerActiveHour") {
        if let Ok(value) = costs.parse::<u32>() {
            this_production.cost_per_hour = value;
        }
    } else if let Some(costs) = xml_node.attribute("costsPerActiveMinute") {
        if let Ok(value) = costs.parse::<u32>() {
            this_production.cost_per_hour = value * 60;
        }
    } else if let Some(costs) = xml_node.attribute("costsPerActiveMonth") {
        if let Ok(value) = costs.parse::<u32>() {
            this_production.cost_per_hour = value / 24;
        }
    }

    if let Some(cycles) = xml_node.attribute("cyclesPerActiveHour") {
        if let Ok(value) = cycles.parse::<u32>() {
            this_production.cycles_per_hour = value;
        }
    } else if let Some(cycles) = xml_node.attribute("cyclesPerActiveMinute") {
        if let Ok(value) = cycles.parse::<u32>() {
            this_production.cycles_per_hour = value * 60;
        }
    } else if let Some(cycles) = xml_node.attribute("cyclesPerActiveMonth") {
        if let Ok(value) = cycles.parse::<u32>() {
            this_production.cycles_per_hour = value / 24;
        }
    }

    for output in xml_node.descendants().filter(|n|n.has_tag_name("output")) {
        // Vec<ProductionIngredient> (all are produced)
        let Some(fill) = output.attribute("fillType") else { continue; };
        let Some(amount) = output.attribute("amount").and_then(|n|n.parse::<f32>().ok())  else { continue; };

        this_production.output.push(ProductionIngredient::new(fill.to_owned().to_lowercase(), amount));
    }

    for input in xml_node.descendants().filter(|n|n.has_tag_name("input")) {
        // fillType and amount always appear for well-formed inputs
        let Some(fill) = input.attribute("fillType") else { continue; };
        let Some(amount) = input.attribute("amount").and_then(|n|n.parse::<f32>().ok())  else { continue; };

        match input.attribute("mix") {
            Some("boost") => {
                // a booster
                let Some(boost_factor) = input.attribute("boostfactor").and_then(|n|n.parse::<f32>().ok())  else { continue; };
                this_production.boosts.push(ProductionBoost::new(fill.to_owned().to_lowercase(), amount, boost_factor));
            },
            Some(index) => {
                //multi-input
                let this_mix = mix_inputs.entry(index.to_owned()).or_default();
                this_mix.push(ProductionIngredient::new(fill.to_owned().to_lowercase(), amount));
            },
            _ => {
                // single
                let this_input:ProductionIngredients = vec![
                    ProductionIngredient::new(fill.to_owned().to_lowercase(), amount)
                ];
                this_production.recipe.push(this_input);
            }
        }
    }

    this_production.recipe.extend(mix_inputs.into_values());

    this_production
}

/// Parse storage (bales and silos)
fn place_parse_storage(xml_tree : &roxmltree::Document, this_place : &mut ModDetailPlace) {
    if let Some(obj_store) = xml_tree.root().children().find(|n|n.has_tag_name("objectStorage")) {
        this_place.storage.objects =  Some(str::parse(obj_store.attribute("capacity").unwrap_or("250")).unwrap_or(250));
    }

    if let Some(silo_node) = xml_tree.descendants().find(|n|n.has_tag_name("silo") || n.has_tag_name("siloExtension")) {
        let mut capacity:Vec<Option<&str>> = vec![];

        for fill_unit in silo_node.descendants().filter(|n|n.has_tag_name("storage")) {    
            capacity.push(fill_unit.attribute("capacity"));
    
            if let Some(cats) = fill_unit.attribute("fillTypeCategories") {
                this_place.storage.silo_fill_cats.extend(cats.split(' ').map(|n|n.to_lowercase().clone()));
            }
            if let Some(cats) = fill_unit.attribute("fillTypes") {
                this_place.storage.silo_fill_types.extend(cats.split(' ').map(|n|n.to_lowercase().clone()));
            }
        }

        this_place.storage.silo_capacity = capacity
            .into_iter()
            .flatten()
            .flat_map(str::parse::<u32>)
            .sum();

        this_place.storage.silo_fill_cats.sort();
        this_place.storage.silo_fill_cats.dedup();
        this_place.storage.silo_fill_types.sort();
        this_place.storage.silo_fill_types.dedup();
    }
}

/// Parse animal husbandry and beehives
fn place_parse_animals(xml_tree : &roxmltree::Document, this_place : &mut ModDetailPlace) {
    if let Some(this_beehive) = xml_tree.descendants().find(|n|n.has_tag_name("beehive")) {
        this_place.animals.beehive_exists = true;
        this_place.animals.beehive_per_day = str::parse(this_beehive.attribute("litersHoneyPerDay").unwrap_or("0")).unwrap_or(0);
        this_place.animals.beehive_radius = str::parse(this_beehive.attribute("actionRadius").unwrap_or("0")).unwrap_or(0);
    }

    if let Some(this_pen) = xml_tree.descendants().find(|n|n.has_tag_name("animals")) {
        this_place.animals.husbandry_exists = true;
        this_place.animals.husbandry_animals = str::parse(this_pen.attribute("maxNumAnimals").unwrap_or("0")).unwrap_or(0);
        this_place.animals.husbandry_type = this_pen.attribute("type").map(std::string::ToString::to_string);
    }
}

/// Parse placeable sorting data
fn place_parse_sorting(xml_tree : &roxmltree::Document, this_place : &mut ModDetailPlace) {
    this_place.sorting.category = xml_extract_text_as_opt_string(xml_tree, "category");
    this_place.sorting.income_per_hour = xml_extract_text_as_opt_u32(xml_tree, "incomePerHour").unwrap_or(0);
    this_place.sorting.name = xml_extract_text_as_opt_string(xml_tree, "name");
    this_place.sorting.price = xml_extract_text_as_opt_u32(xml_tree, "price").unwrap_or(0);
    this_place.sorting.type_name = xml_tree.root_element().attribute("type").map(std::string::ToString::to_string);

    if xml_tree.descendants().filter(|n|n.has_tag_name("color")).count() > 1 {
        this_place.sorting.has_color = VehicleCapability::Yes;
    }

    this_place.sorting.functions = xml_tree.descendants()
        .filter(|n|n.has_tag_name("function"))
        .filter_map(|n|n.text())//(|n|Some(n))
        .map(std::string::ToString::to_string)
        .collect();
}
