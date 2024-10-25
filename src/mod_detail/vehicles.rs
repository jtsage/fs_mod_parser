//! Parse vehicles
use crate::ModParserOptions;
use crate::mod_detail::structs::{VehicleCapability, ModDetailVehicle, ModDetailSprayType};
use crate::shared::files::AbstractFileHandle;
use crate::mod_detail::structs::{MotorEntry, MotorValue};
use super::{xml_extract_text_as_opt_string, xml_extract_text_as_opt_u32};
use crate::shared::{extract_and_normalize_image, convert_mod_icon};
use std::f32::consts::PI;

/// Parse a vehicle
/// 
/// # Sample Output
/// ```json
///{
///    "fillSpray": {
///        "fillCat": [],
///        "fillLevel": 11433,
///        "fillType": [ "liquidfertilizer", "seeds" ],
///        "sprayTypes": [
///            {
///                "fills": [ "fertilizer" ],
///                "width": null
///            }
///        ]
///    },
///    "flags": {
///        "beacons": false,
///        "color": false,
///        "enterable": false,
///        "lights": true,
///        "motorized": false,
///        "wheels": true
///    },
///    "iconBase": null,
///    "iconFile": null,
///    "masterType": "vehicle",
///    "motor": {
///        "fuelType": null,
///        "transmissionType": null,
///        "motors": []
///    },
///    "sorting": {
///        "brand": "JOHNDEERE",
///        "category": "planters",
///        "combos": [],
///        "name": "1775NT 2022",
///        "typeName": "fertilizingSowingMachine",
///        "typeDescription": "$l10n_typeDesc_sowingMachine",
///        "year": null
///    },
///    "specs": {
///        "functions": [
///            "$l10n_function_planter",
///            "$l10n_function_sowingMachineDirect"
///        ],
///        "jointAccepts": [ "trailer", "trailerLow" ],
///        "jointRequires": [ "implement" ],
///        "name": "1775NT 2022",
///        "price": 362878,
///        "specs": {
///            "neededPower": 340,
///            "speedLimit": 16
///        },
///        "weight": 6900
///    }
///}
/// ```
pub fn vehicle_parse(xml_tree : &roxmltree::Document, file_handle: &mut Box<dyn AbstractFileHandle>,  options : &ModParserOptions ) -> ModDetailVehicle {
    let mut this_vehicle = ModDetailVehicle::new();
    
    vehicle_parse_sorting(xml_tree, &mut this_vehicle);
    vehicle_parse_flags(xml_tree, &mut this_vehicle);
    vehicle_parse_specs(xml_tree, &mut this_vehicle);
    vehicle_parse_fills(xml_tree, &mut this_vehicle);
    vehicle_parse_motor(xml_tree, &mut this_vehicle);

    if !options.skip_detail_icons {
        let image_entry = extract_and_normalize_image(xml_tree, "image");

        if let Some(filename) = image_entry.base_game {
            this_vehicle.icon_base = Some(filename);
        } else if let Some(filename) = image_entry.local_file {
            if let Ok(file_content) = file_handle.as_bin(&filename) {
                this_vehicle.icon_file = convert_mod_icon(file_content);
            }
        }
    }

    this_vehicle
}

/// Transient motor torque entry
struct TorqueEntry {
    /// Torque
    pub torque   : f32,
    /// motor RPM
    pub rpm      : f32
}

impl TorqueEntry {
    /// Create new torque entry
    fn new(node : &roxmltree::Node, motor_rpm : f32) -> Self {
        let norm_rpm = node
            .attribute("normRpm")
            .map_or(1_f32, |n|n.parse::<f32>().unwrap_or(1_f32));

        TorqueEntry {
            torque : node
                .attribute("torque")
                .map_or(1_f32, |n|n.parse::<f32>().unwrap_or(1_f32)),
            rpm : node
                .attribute("rpm")
                .map_or(motor_rpm * norm_rpm, |n|n.parse::<f32>().unwrap_or(motor_rpm * norm_rpm))
        }
    }
}

/// Parse motor configurations
fn vehicle_parse_motor(xml_tree : &roxmltree::Document, this_vehicle : &mut ModDetailVehicle) {
    let mut torque_entries: Vec<TorqueEntry> = vec![];
    let mut motor_rpm = 1800_f32;
    let mut transmission_name = "";
    let mut min_fwd_gear_and_axel_ratio = f32::MAX;

    for motor_config in xml_tree.descendants().filter(|n|n.has_tag_name("motorConfiguration")) {
        let Some(motor_entry) = motor_config.children().find(|n|n.has_tag_name("motor")) else { continue; };

        // Get current motor RPM, or use last, or use default of 1800
        if let Some(max_rpm) = motor_entry.attribute("maxRpm") {
            if let Ok(max_rpm_f) = max_rpm.parse::<f32>() {
                motor_rpm = max_rpm_f;
            }
        }

        let motor_scale = motor_entry
            .attribute("torqueScale")
            .map_or(1_f32, |n|n.parse::<f32>().unwrap_or(1_f32));

        // If new torque entries exist, replace the "last" list
        let mut torque_iter = motor_config.descendants().filter(|n|n.has_tag_name("torque")).peekable();

        if torque_iter.peek().is_some() {
            torque_entries.clear();
            for torque_node in torque_iter {
                torque_entries.push(TorqueEntry::new(&torque_node, motor_rpm));
            }
        }

        // Check for a transmission definition
        if let Some(new_transmission) = motor_config.children().find(|n|n.has_tag_name("transmission")) {
            // Invalidate the old ratio
            min_fwd_gear_and_axel_ratio = f32::MAX;

            // New name found, overwrite "last"
            if let Some(trans_name) = new_transmission.attribute("name") {
                transmission_name = trans_name;
                if this_vehicle.motor.transmission_type.is_none() {
                    this_vehicle.motor.transmission_type = Some(transmission_name.to_owned());
                }
            }

            let axel_ratio = new_transmission
                .attribute("axleRatio")
                .map_or(1_f32, |n|n.parse::<f32>().unwrap_or(1_f32));

            if let Some(fwd_gear_ratio) = new_transmission.attribute("minForwardGearRatio") {
                // found minForwardGearRatio, can calculate `min_fwd_gear_and_axel_ratio`
                min_fwd_gear_and_axel_ratio = axel_ratio * fwd_gear_ratio.parse::<f32>().unwrap_or(1_f32);
            } else {
                // we have to calculate the ratio
                for forward_gear in new_transmission.children().filter(|n|n.has_tag_name("forwardGear")) {
                    if let Some(known_ratio) = forward_gear.attribute("gearRatio") {
                        min_fwd_gear_and_axel_ratio = f32::min(
                            min_fwd_gear_and_axel_ratio, 
                            axel_ratio * known_ratio.parse::<f32>().unwrap_or(1_f32)
                        );
                    } else if let Some(known_max) = forward_gear.attribute("maxSpeed") {
                        min_fwd_gear_and_axel_ratio = f32::min(
                            min_fwd_gear_and_axel_ratio, 
                            axel_ratio * (motor_rpm * PI / ( known_max.parse::<f32>().unwrap_or(1_f32) / 3.6_f32 * 30_f32 ))
                        );
                    }
                }
            }
        } // end new transmission

        // Get defined max speed for the motor
        let defined_max_speed = motor_entry
            .attribute("maxForwardSpeed")
            .map_or(0, |n|n.parse::<u32>().unwrap_or(0));

        let motor_name = motor_config
            .attribute("name")
            .unwrap_or("--");

        let motor_hp_name = motor_config
            .attribute("hp")
            .unwrap_or("");

        let mut full_name = String::from(motor_name);
        if ! transmission_name.is_empty() {
            full_name.push(' ');
            full_name.push_str(transmission_name);
        }
        if ! motor_hp_name.is_empty() {
            full_name.push(' ');
            full_name.push_str(motor_hp_name);
        }

        let mut motor_record = MotorEntry::new(full_name, defined_max_speed);

        for torque_entry in &torque_entries {
            motor_record.horse_power.push(MotorValue::new(
                torque_entry.rpm,
                motor_scale * ( 1.359_621_6 * PI * torque_entry.rpm * torque_entry.torque ) / 30.0
            ));
            motor_record.speed_kph.push(MotorValue::new(
                torque_entry.rpm,
                3.6 * ( ( torque_entry.rpm * PI ) / ( 30.0 * min_fwd_gear_and_axel_ratio ) )
            ));
            motor_record.speed_mph.push(MotorValue::new(
                torque_entry.rpm,
                3.6 * ( ( torque_entry.rpm * PI ) / ( 30.0 * min_fwd_gear_and_axel_ratio ) * 0.621_371 )
            ));
        }
        this_vehicle.motor.motors.push(motor_record);
    } // end motor_config

    this_vehicle.motor.fuel_type = xml_tree
        .descendants()
        .find(|n|n.has_attribute("consumer"))
        .and_then(|n|n.attribute("fillType"))
        .map(std::string::ToString::to_string);
}

/// Parse fill levels
fn vehicle_parse_fills(xml_tree : &roxmltree::Document, this_vehicle : &mut ModDetailVehicle) {
    let mut capacity:Vec<Option<&str>> = vec![];

    for fill_unit in xml_tree.descendants().filter(|n|n.has_tag_name("fillUnit") && (n.has_attribute("fillTypes") || n.has_attribute("fillTypeCategories"))) {
        if let Some(skipper) = fill_unit.attribute("showInShop") {
            if skipper == "false" { continue; }
        }

        capacity.push(fill_unit.attribute("capacity"));

        if let Some(cats) = fill_unit.attribute("fillTypeCategories") {
            this_vehicle.fill_spray.fill_cat.extend(cats.split(' ').map(|n|n.to_lowercase().clone()));
        }
        if let Some(cats) = fill_unit.attribute("fillTypes") {
            this_vehicle.fill_spray.fill_type.extend(cats.split(' ').map(|n|n.to_lowercase().clone()));
        }
    }

    this_vehicle.fill_spray.fill_level = capacity
        .into_iter()
        .flatten()
        .flat_map(str::parse::<u32>)
        .sum();

    this_vehicle.fill_spray.fill_cat.sort();
    this_vehicle.fill_spray.fill_cat.dedup();
    this_vehicle.fill_spray.fill_type.sort();
    this_vehicle.fill_spray.fill_type.dedup();

    for spray_type in xml_tree.descendants().filter(|n|n.has_tag_name("sprayType")) {
        this_vehicle.fill_spray.spray_types.push(ModDetailSprayType{

            width : spray_type
                .children()
                .find(|n|n.has_tag_name("usageScales"))
                .and_then(|n|n.attribute("workingWidth"))
                .and_then(|n|n.parse::<f32>().ok()),

            fills : spray_type
                .attribute("fillTypes")
                .map_or(vec![], |n| n
                    .split(' ')
                    .filter(|n|*n!="unknown")
                    .map(|n|n.to_lowercase().clone())
                    .collect()
                )
        });
    }
}

/// Parse vehicle sorting info
fn vehicle_parse_sorting(xml_tree : &roxmltree::Document, this_vehicle : &mut ModDetailVehicle) {
    this_vehicle.sorting.name = xml_extract_text_as_opt_string(xml_tree, "name");
    this_vehicle.sorting.brand = xml_extract_text_as_opt_string(xml_tree, "brand");
    this_vehicle.sorting.category = xml_extract_text_as_opt_string(xml_tree, "category");
    this_vehicle.sorting.type_description = xml_extract_text_as_opt_string(xml_tree, "typeDesc");
    this_vehicle.sorting.type_name = xml_tree.root_element().attribute("type").map(std::string::ToString::to_string);
    this_vehicle.sorting.year = xml_extract_text_as_opt_u32(xml_tree, "year");

    this_vehicle.sorting.combos = xml_tree.descendants()
        .filter(|n| n.has_tag_name("combination"))
        .filter_map(|n|n.attribute("xmlFilename"))
        .map(std::string::ToString::to_string)
        .collect();
}

/// Parse vehicle flags
fn vehicle_parse_flags(xml_tree : &roxmltree::Document, this_vehicle : &mut ModDetailVehicle) {
    if xml_tree.descendants().any(|n|n.has_tag_name("beaconLights")) {
        this_vehicle.flags.beacons = VehicleCapability::Yes;
    }
    if xml_tree.descendants().any(|n|n.has_tag_name("baseMaterialConfiguration")) {
        this_vehicle.flags.color = VehicleCapability::Yes;
    }
    if xml_tree.descendants().any(|n|n.has_tag_name("enterable")) {
        this_vehicle.flags.enterable = VehicleCapability::Yes;
    }
    if xml_tree.descendants().any(|n|n.has_tag_name("realLights")) {
        this_vehicle.flags.lights = VehicleCapability::Yes;
    }
    if xml_tree.descendants().any(|n|n.has_tag_name("motorized")) {
        this_vehicle.flags.motorized = VehicleCapability::Yes;
    }
    if xml_tree.descendants().filter(|n|n.has_tag_name("wheelConfiguration")).count() > 1 {
        this_vehicle.flags.wheels = VehicleCapability::Yes;
    }
}

/// Parse vehicle specs
fn vehicle_parse_specs(xml_tree : &roxmltree::Document, this_vehicle : &mut ModDetailVehicle) {
    if let Some(node) = xml_tree.descendants().find(|n| n.has_tag_name("speedLimit")) {
        if let Some(value) = node
            .attribute("value")
            .and_then(|n|n.parse::<u32>().ok()) {
                this_vehicle.specs.specs.insert(String::from("speedLimit"), value);
        }
    }

    if let Some(spec_node) = xml_tree.descendants().find(|n|n.has_tag_name("specs")) {
        for spec in spec_node.children().filter(|n| !n.has_tag_name("combination")) {
            if let Some(value) = spec.text().and_then(|n|n.parse::<u32>().ok()) {
                this_vehicle.specs.specs.insert(
                    spec.tag_name().name().to_owned(),
                    value
                );
            }
        }
    }

    this_vehicle.specs.price = xml_extract_text_as_opt_u32(xml_tree, "price").unwrap_or(0);
    this_vehicle.specs.name  = xml_extract_text_as_opt_string(xml_tree, "name").unwrap_or(String::from("--"));

    this_vehicle.specs.functions = xml_tree.descendants()
        .filter(|n|n.has_tag_name("function"))
        .filter_map(|n|n.text())//(|n|Some(n))
        .map(std::string::ToString::to_string)
        .collect();

    this_vehicle.specs.weight = xml_tree.descendants()
        .filter(|n| n.has_tag_name("component"))
        .filter_map(|n| n.attribute("mass"))
        .filter_map(|n| n.parse::<u32>().ok() )
        .sum::<u32>();

    this_vehicle.specs.joint_accepts =  xml_tree.descendants()
        .filter(|n|n.has_tag_name("attacherJoint"))
        .filter_map(|n|n.attribute("jointType"))
        .map(std::string::ToString::to_string)
        .collect();

    this_vehicle.specs.joint_accepts.sort();
    this_vehicle.specs.joint_accepts.dedup();

    this_vehicle.specs.joint_requires =  xml_tree.descendants()
        .filter(|n|n.has_tag_name("inputAttacherJoint"))
        .filter_map(|n|n.attribute("jointType"))
        .map(std::string::ToString::to_string)
        .collect();

    this_vehicle.specs.joint_requires.sort();
    this_vehicle.specs.joint_requires.dedup();
}
