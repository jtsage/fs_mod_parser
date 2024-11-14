use assert_json_diff::{assert_json_eq, assert_json_include};
use fs_mod_parser::mod_detail::structs::{ModDetail, ModDetailError};
use fs_mod_parser::{parse_detail, parse_mod_with_options, parse_detail_with_options, ModParserOptions};
use serde_json::json;
use std::collections::HashSet;
use std::path::Path;

static NO_ICONS: ModParserOptions = ModParserOptions {
    include_mod_detail: true,
    include_save_game: false,
    skip_detail_icons: true,
    skip_mod_icons: false,
};

static PATH_TO_GOOD: &str = "./tests/test_mods/DETAIL_Samples.zip";
static PATH_TO_BAD: &str = "./tests/test_mods/DETAIL_Internal_Failures.zip";

#[test]
fn missing_file() {
    let test_file_path = Path::new("./tests/test_mods/DETAIL_Fake_File.zip");
    assert!(!test_file_path.exists());

    let detail_record = parse_detail(test_file_path);

    let expected_errors: HashSet<ModDetailError> = HashSet::from([ModDetailError::FileReadFail]);
    assert_eq!(detail_record.issues, expected_errors);

    assert!(detail_record.to_json().len() > 10);
    assert!(detail_record.to_json_pretty().len() > 10);
}

#[test]
fn invalid_folder() {
    let test_file_path = Path::new("./tests/test_mods/FAILURE_Invalid_Folder");
    assert!(test_file_path.exists());

    let detail_record = parse_detail(test_file_path);

    let expected_errors: HashSet<ModDetailError> = HashSet::from([ModDetailError::NotModModDesc]);
    assert_eq!(detail_record.issues, expected_errors);

    assert!(detail_record.to_json().len() > 10);
    assert!(detail_record.to_json_pretty().len() > 10);
}

#[test]
fn invalid_moddesc_folder() {
    let test_file_path = Path::new("./tests/test_mods/FAILURE_Invalid_ModDesc");
    assert!(test_file_path.exists());

    let detail_record = parse_detail(test_file_path);

    let expected_errors: HashSet<ModDetailError> = HashSet::from([ModDetailError::NotModModDesc]);
    assert_eq!(detail_record.issues, expected_errors);

    assert!(detail_record.to_json().len() > 10);
    assert!(detail_record.to_json_pretty().len() > 10);
}

#[test]
fn good_store_items_overview() {
    let test_file_path = Path::new(PATH_TO_GOOD);
    assert!(test_file_path.exists());

    let mod_record = parse_mod_with_options(test_file_path, &NO_ICONS);
    let mod_record_json = mod_record.to_json_pretty().clone();

    let detail_record = &mod_record.include_detail.unwrap();

    assert_eq!(detail_record.issues.len(), 0);
    assert_eq!(detail_record.brands.len(), 2);
    assert_eq!(detail_record.l10n.len(), 2);
    assert_eq!(detail_record.placeables.len(), 3);
    assert_eq!(detail_record.vehicles.len(), 4);

    /* cSpell: disable */
    let expect_brand = HashSet::from([
        String::from("KRAMPE"),
        String::from("JOHNDEERE"),
        String::from("FENDT"),
        String::from("CASEIH"),
    ]);
    let expect_cat = HashSet::from([
        String::from("fertilizerSpreaders"),
        String::from("productionPoints"),
        String::from("trailers"),
        String::from("animalpens"),
        String::from("tractorsL"),
        String::from("harvesters"),
    ]);
    /* cSpell: enable */
    assert_eq!(detail_record.item_brands, expect_brand);
    assert_eq!(detail_record.item_categories, expect_cat);

    let byte_length = mod_record_json.len() as i32;
    let byte_expected: i32 = 32075;
    let byte_margin = 100;
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


#[test]
fn good_store_items_overview_full() {
    let test_file_path = Path::new(PATH_TO_GOOD);
    assert!(test_file_path.exists());

    let detail_record = parse_detail(test_file_path);
    let _ = detail_record.to_json();

    assert_eq!(detail_record.issues.len(), 0);
    assert_eq!(detail_record.brands.len(), 2);
    assert_eq!(detail_record.l10n.len(), 2);
    assert_eq!(detail_record.placeables.len(), 3);
    assert_eq!(detail_record.vehicles.len(), 4);

    /* cSpell: disable */
    let expect_brand = HashSet::from([
        String::from("KRAMPE"),
        String::from("JOHNDEERE"),
        String::from("FENDT"),
        String::from("CASEIH"),
    ]);
    let expect_cat = HashSet::from([
        String::from("fertilizerSpreaders"),
        String::from("productionPoints"),
        String::from("trailers"),
        String::from("animalpens"),
        String::from("tractorsL"),
        String::from("harvesters"),
    ]);
    /* cSpell: enable */
    assert_eq!(detail_record.item_brands, expect_brand);
    assert_eq!(detail_record.item_categories, expect_cat);

    let byte_length = detail_record.to_json_pretty().len() as i32;
    let byte_expected: i32 = 108433;
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
    if let Some(comp_key) = setup_good_store_items()
        .placeables
        .get("xml/place-husbandry.xml")
    {
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
            "iconOrig": null,
            "masterType": "placeable",
            "parentItem": null,
            "productions": [],
            "showInStore": true,
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
    } else {
        panic!("key not found");
    };
}

#[test]
fn good_place_deep_production() {
    /* cSpell: disable */
    if let Some(comp_key) = setup_good_store_items()
        .placeables
        .get("xml/production-deep.xml")
    {
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
                "costPerHour": 2.0,
                "cyclesPerHour": 10.0,
                "name": "$l10n_FS22_ProductionRevamp_Productions_productline_pigfood",
                "output": [ { "amount": 1000.0, "fillType": "pigfood" } ],
                "params": "",
            },
            {
                "boosts": [],
                "costPerHour": 2.0,
                "cyclesPerHour": 8.0,
                "name": "$l10n_FS22_ProductionRevamp_Productions_productline_forage",
                "output": [ { "amount": 2000.0, "fillType": "forage" } ],
                "params": "",
            },
            {
                "boosts": [],
                "costPerHour": 2.0,
                "cyclesPerHour": 8.0,
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
    } else {
        panic!("key not found");
    };
}

#[test]
fn good_place_simple_production() {
    /* cSpell: disable */
    if let Some(comp_key) = setup_good_store_items()
        .placeables
        .get("xml/production-simple.xml")
    {
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
                "costPerHour": 2.0,
                "cyclesPerHour": 1.0,
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
    } else {
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
    if let Some(comp_key) = setup_good_store_items()
        .vehicles
        .get("xml/example-fillunit.xml")
    {
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
    } else {
        panic!("key not found");
    };
}

#[test]
fn good_vehicle_sprayer_types() {
    /* cSpell: disable */
    if let Some(comp_key) = setup_good_store_items()
        .vehicles
        .get("xml/example-spraytypes.xml")
    {
        let actual = json!(comp_key);
        let expected = json!({
            "fillSpray": {
                "fillCat": [],
                "fillLevel": 15000,
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
    } else {
        panic!("key not found");
    };
}

#[test]
fn good_vehicle_multiple_motors() {
    /* cSpell: disable */
    if let Some(comp_key) = setup_good_store_items()
        .vehicles
        .get("xml/example-multimotor.xml")
    {
        assert_eq!(
            comp_key.motor.fuel_type,
            Some(String::from("electricCharge"))
        );
        assert_eq!(
            comp_key.motor.transmission_type,
            Some(String::from("$l10n_info_transmission_cvt"))
        );
        assert_eq!(comp_key.motor.motors.len(), 4);
    } else {
        panic!("key not found");
    };
}

#[test]
fn bad_store_items_overview() {
    let test_file_path = Path::new(PATH_TO_BAD);
    assert!(test_file_path.exists());

    let detail_record = parse_detail(test_file_path);
    let _ = detail_record.to_json();

    let expected_errors: HashSet<ModDetailError> = HashSet::from([
        ModDetailError::BrandMissingIcon,
        ModDetailError::StoreItemBroken,
        ModDetailError::StoreItemMissing,
    ]);
    assert_eq!(detail_record.issues, expected_errors);

    assert_eq!(detail_record.brands.len(), 2);
    assert_eq!(detail_record.l10n.len(), 2);
    assert_eq!(detail_record.placeables.len(), 0);
    assert_eq!(detail_record.vehicles.len(), 0);

    let byte_length = detail_record.to_json_pretty().len() as i32;
    let byte_expected: i32 = 1497;
    let byte_margin = 100;
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


#[test]
fn good_vehicle_parent_item() {
    /* cSpell: disable */
    if let Some(comp_key) = setup_good_store_items()
        .vehicles
        .get("xml/vehicle-with-parent.xml")
    {
        let actual = json!(comp_key);
        let expected = json!({
            "fillSpray": {
                "fillCat": [],
                "fillLevel": 0,
                "fillType": [],
                "sprayTypes": []
            },
            "flags": {
                "beacons": false,
                "color": false,
                "enterable": false,
                "lights": false,
                "motorized": false,
                "wheels": false
            },
            "iconBase": null,
            "iconFile": null,
            "iconOrig": null,
            "parentItem": "$data/vehicles/fendt/ideal/ideal.xml",
            "masterType": "vehicle",
            "motor": {
                "fuelType": null,
                "transmissionType": null,
                "motors": []
            },
            "sorting": {
                "brand": "FENDT",
                "category": "harvesters",
                "combos": [],
                "name": "IDEAL ParaLevel",
                "typeName": "combineDrivable",
                "typeDescription": null,
                "year": null
            },
            "specs": {
                "functions": [],
                "jointAccepts": [],
                "jointRequires": [],
                "name": "IDEAL ParaLevel",
                "price": 0,
                "specs": {},
                "weight": 0
            }
        });
        /* cSpell: enable */
        assert_json_include!(actual : actual, expected : expected);
    } else {
        panic!("key not found");
    };
}