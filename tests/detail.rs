use std::collections::HashSet;
use std::path::Path;
use fs_mod_parser::{parse_detail, parse_detail_with_options, ModParserOptions};
use fs_mod_parser::mod_detail::structs::{ModDetailError, ModDetail};
use assert_json_diff::{assert_json_include, assert_json_eq};
use serde_json::json;

static NO_ICONS:ModParserOptions = ModParserOptions {
    include_mod_detail : true,
    include_save_game  : false,
    skip_detail_icons  : true,
    skip_mod_icons     : false,
};

static PATH_TO_GOOD: &str = "./tests/test_mods/DETAIL_Samples.zip";

#[test]
fn missing_file() {
    let test_file_path = Path::new("./tests/test_mods/DETAIL_Fake_File.zip");
    assert!(!test_file_path.exists());

    let detail_record = parse_detail(test_file_path);

    let expected_errors:HashSet<ModDetailError> = HashSet::from([ModDetailError::FileReadFail]);
    assert_eq!(detail_record.issues, expected_errors);

    assert!(detail_record.to_json().len() > 10);
    assert!(detail_record.to_json_pretty().len() > 10);
}

#[test]
fn good_store_items_overview() {
    let test_file_path = Path::new(PATH_TO_GOOD);
    assert!(test_file_path.exists());

    let detail_record = parse_detail(test_file_path);
    let _ = detail_record.to_json();

    assert_eq!(detail_record.issues.len(), 0);
    assert_eq!(detail_record.brands.len(), 2);
    assert_eq!(detail_record.l10n.len(), 2);
    assert_eq!(detail_record.placeables.len(), 3);
    assert_eq!(detail_record.vehicles.len(), 3);

    let byte_length = detail_record.to_json_pretty().len() as i32;
	let byte_expected:i32 = 106552;
	let byte_margin = 500;
	assert!(
		(byte_length - byte_expected).abs() < byte_margin,
		"assertion failed: `(left !== right)` \
		(left: `{:?}`, right: `{:?}`, expect diff: `{:?}`, real diff: `{:?}`)",
		byte_length,
		byte_expected,
		byte_margin,
		(byte_length - byte_expected).abs()
	);
}

fn setup_good_store_items() -> ModDetail {
    let test_file_path = Path::new(PATH_TO_GOOD);
    assert!(test_file_path.exists());

    let detail_record = parse_detail_with_options(test_file_path, &NO_ICONS);
    detail_record
}

#[test]
fn good_store_l10n() {
    /* cSpell: disable */
    let actual = json!(setup_good_store_items());
    let expected = json!({
        "l10n": {
            "en": {
                "colorconfigjts_bronze": "Metallic Bronze",
                "colorconfigjts_copper": "Metallic Copper",
                "colorconfigjts_silver_steel": "Silver Steel",
                "colorconfigjts_galv_steel": "Galvanized Steel",
                "colorconfigjts_gold": "Metallic Gold",
                "colorconfigjts_brush_silver": "Bright Brushed Steel",
                "colorconfigjts_brush_steel": "Dark Brushed Steel",
                "colorconfigjts_black_steel": "Black Steel"
            },
            "de": {
                "colorconfigjts_gold": "Gold metallic",
                "colorconfigjts_silver_steel": "Silberner Stahl",
                "colorconfigjts_bronze": "Bronze metallic",
                "colorconfigjts_black_steel": "Schwarzer Stahl",
                "colorconfigjts_brush_silver": "Blank gebürsteter Stahl",
                "colorconfigjts_brush_steel": "Dunkel gebürsteter Stahl",
                "colorconfigjts_copper": "Kupfer metallic",
                "colorconfigjts_galv_steel": "Verzinkter Stahl"
            }
        }
    });
    /* cSpell: enable */
    assert_json_include!(actual : actual, expected : expected);
}

#[test]
fn good_place_husbandry() {
    /* cSpell: disable */
    if let Some (comp_key) = setup_good_store_items().placeables.get("xml/place-husbandry.xml") {
        let actual = json!(comp_key);
        let expected = json!({
            "animals": {
                "beehiveExists": false,
                "beehivePerDay": 0,
                "beehiveRadius": 0,
                "husbandryAnimals": 5000,
                "husbandryExists": true,
                "husbandryType": "CHICKEN"
            },
            "iconBase": null,
            "iconFile": null,
            "masterType": "placeable",
            "productions": [],
            "sorting": {
                "category": "animalpens",
                "functions": [
                    "$l10n_function_animalPenChicken"
                ],
                "hasColor": true,
                "incomePerHour": 0,
                "name": "$l10n_storeItem_animalBarnChickenBig",
                "price": 150000,
                "typeName": "chickenHusbandry"
            },
            "storage": {
                "objects": null,
                "siloCapacity": 0,
                "siloExists": false,
                "siloFillCats": [],
                "siloFillTypes": []
            }
        });
        /* cSpell: enable */
        assert_json_eq!(actual, expected);
    }else {
        panic!("key not found");
    };
}

#[test]
fn good_place_deep_production() {
    /* cSpell: disable */
    if let Some (comp_key) = setup_good_store_items().placeables.get("xml/production-deep.xml") {
        let actual = json!(comp_key);
        let expected = json!({
            "animals": {
                "beehiveExists": false,
                "beehivePerDay": 0,
                "beehiveRadius": 0,
                "husbandryAnimals": 0,
                "husbandryExists": false,
                "husbandryType": null
            },
            "iconBase": null,
            "iconFile": null,
            "masterType": "placeable",
            "productions": [
            {
                "boosts": [],
                "costPerHour": 2,
                "cyclesPerHour": 10,
                "name": "$l10n_FS22_ProductionRevamp_Productions_productline_pigfood",
                "output": [ { "amount": 1000.0, "fillType": "pigfood" } ],
                "params": "",
            },
            {
                "boosts": [],
                "costPerHour": 2,
                "cyclesPerHour": 8,
                "name": "$l10n_FS22_ProductionRevamp_Productions_productline_forage",
                "output": [ { "amount": 2000.0, "fillType": "forage" } ],
                "params": "",
            },
            {
                "boosts": [],
                "costPerHour": 2,
                "cyclesPerHour": 8,
                "name": "$l10n_FS22_ProductionRevamp_Productions_productline_drygrass",
                "output": [ { "amount": 1000.0, "fillType": "drygrass_windrow" } ],
                "params": "",
            }
            ],
            "sorting": {
                "category": "productionPoints",
                "functions": [ "$l10n_FS22_ProductionRevamp_Productions_farma800_function" ],
                "hasColor": false,
                "incomePerHour": 0,
                "name": "$l10n_FS22_ProductionRevamp_Productions_shopItem_farma800",
                "price": 60000,
                "typeName": "productionPoint"
            },
            "storage": {
                "objects": null,
                "siloCapacity": 0,
                "siloExists": false,
                "siloFillCats": [],
                "siloFillTypes": []
            }
        });
        /* cSpell: enable */
        assert_json_include!(actual : actual, expected : expected);
    }else {
        panic!("key not found");
    };
}

#[test]
fn good_place_simple_production() {
    /* cSpell: disable */
    if let Some (comp_key) = setup_good_store_items().placeables.get("xml/production-simple.xml") {
        let actual = json!(comp_key);
        let expected = json!({
            "animals": {
                "beehiveExists": false,
                "beehivePerDay": 0,
                "beehiveRadius": 0,
                "husbandryAnimals": 0,
                "husbandryExists": false,
                "husbandryType": null
            },
            "iconBase": null,
            "iconFile": null,
            "masterType": "placeable",
            "productions": [
            {
                "boosts": [ { "amount": 1.0, "fillType": "silage_additive" } ],
                "costPerHour": 2,
                "cyclesPerHour": 1,
                "name": "$l10n_FS22_ProductionRevamp_Productions_productline_silage",
                "output": [ { "amount": 40000.0, "fillType": "silage" } ],
                "params": "",
                "recipe": [
                    [
                        { "amount": 40000.0, "fillType": "chaff" },
                        { "amount": 40000.0, "fillType": "grass_windrow" },
                        { "amount": 40000.0, "fillType": "straw" }
                    ]
                ]
                }
            ],
            "sorting": {
                "category": "productionPoints",
                "functions": [ "$l10n_FS22_ProductionRevamp_Productions_farma250-3_function" ],
                "hasColor": false,
                "incomePerHour": 0,
                "name": "$l10n_FS22_ProductionRevamp_Productions_shopItem_farma250-3",
                "price": 45000,
                "typeName": "productionPoint"
            },
            "storage": {
                "objects": null,
                "siloCapacity": 0,
                "siloExists": false,
                "siloFillCats": [],
                "siloFillTypes": []
            }
        });
        /* cSpell: enable */
        assert_json_include!(actual : actual, expected : expected);
    }else {
        panic!("key not found");
    };
}

#[test]
fn good_store_brands() {
    let test_file_path = Path::new(PATH_TO_GOOD);
    assert!(test_file_path.exists());

    let detail_record = parse_detail(test_file_path);
    let actual = json!(detail_record);

    /* cSpell: disable */
    let expected = json!({
        "brands": {
            "HONEYBEE": {
                "title": "Honey Bee",
                "iconBase": null
            },
            "LIZARDLOGISTICS": {
                "title": "Lizard Logistics",
                "iconFile": null,
                "iconBase": "$data/store/brands/brand_lizardLogistics.dds"
            }
        }
    });
    /* cSpell: enable */

    assert!(detail_record.brands["HONEYBEE"].icon_file.is_some());
    assert_json_include!(actual : actual, expected : expected);

}


#[test]
fn good_vehicle_fill_unit() {
    /* cSpell: disable */
    if let Some (comp_key) = setup_good_store_items().vehicles.get("xml/example-fillunit.xml") {
        let actual = json!(comp_key);
        let expected = json!({
            "fillSpray": {
                "fillCat": [ "bulk" ],
                "fillLevel": 59400,
                "fillType": [],
                "sprayTypes": []
            },
            "flags": {
                "beacons": false,
                "color": true,
                "enterable": false,
                "lights": true,
                "motorized": false,
                "wheels": false
            },
            "iconBase": null,
            "iconFile": null,
            "masterType": "vehicle",
            "motor": {
                "fuelType": null,
                "transmissionType": null,
                "motors": []
            },
            "sorting": {
                "brand": "KRAMPE",
                "category": "trailers",
                "combos": [ "$data/vehicles/krampe/dolly10L/dolly10L.xml" ],
                "name": "SKS 30/1050",
                "typeName": "trailer",
                "typeDescription": "$l10n_typeDesc_tipper",
                "year": null
            },
            "specs": {
                "functions": [
                    "$l10n_function_tipper",
                    "$l10n_function_semiTrailer"
                ],
                "jointAccepts": [],
                "jointRequires": [ "semitrailer" ],
                "name": "SKS 30/1050",
                "price": 95000,
                "specs": {},
                "weight": 6870
            }
        });
        /* cSpell: enable */
        assert_json_include!(actual : actual, expected : expected);
    }else {
        panic!("key not found");
    };
}

#[test]
fn good_vehicle_sprayer_types() {
    /* cSpell: disable */
    if let Some (comp_key) = setup_good_store_items().vehicles.get("xml/example-spraytypes.xml") {
        let actual = json!(comp_key);
        let expected = json!({
            "fillSpray": {
                "fillCat": [],
                "fillLevel": 26000,
                "fillType": [ "fertilizer", "lime", "seeds" ],
                "sprayTypes": [
                    { "fills": [ "fertilizer" ] },
                    { "fills": [ "lime" ] }
                ]
            },
            "flags": {
                "beacons": false,
                "color": true,
                "enterable": false,
                "lights": true,
                "motorized": false,
                "wheels": false
            },
            "iconBase": null,
            "iconFile": null,
            "masterType": "vehicle",
            "motor": {
                "fuelType": null,
                "transmissionType": null,
                "motors": []
            },
            "sorting": {
                "brand": "CASEIH",
                "category": "fertilizerSpreaders",
                "combos": [ "casetitan_3000.xml", "casetitan_4000.xml" ],
                "name": "FA 810 Air Boom",
                "typeName": "sprayer",
                "typeDescription": "$l10n_typeDesc_fertilizerSpreader",
                "year": null
            },
            "specs": {
                "functions": [ "$l10n_function_fertilizer" ],
                "jointAccepts": [],
                "jointRequires": [ "titan810" ],
                "name": "FA 810 Air Boom",
                "price": 95000,
                "specs": {
                    "workingWidth": 18,
                    "speedLimit": 30
                },
                "weight": 3000
            }
        });
        /* cSpell: enable */
        assert_json_include!(actual : actual, expected : expected);
    }else {
        panic!("key not found");
    };
}

// TODO: add multi-motor
// TODO: add more error types "one or more xml's failed", "one or more icons failed"