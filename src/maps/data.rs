//! Base game data
use super::structs::{Crop, CropSeason, CropTypeState};

/// Crop types to ignore
pub const SKIP_CROP_TYPES: [&str; 2] = ["meadow", "unknown"];

/// Basegame supplied crop growth definitions, FS22
pub const BG_CROP_TYPES: [CropTypeState; 17] = [
    CropTypeState {
        name: "wheat",
        max_harvest: 8,
        min_harvest: 8,
        states: 8,
    },
    CropTypeState {
        name: "barley",
        max_harvest: 7,
        min_harvest: 7,
        states: 7,
    },
    CropTypeState {
        name: "canola",
        max_harvest: 9,
        min_harvest: 9,
        states: 9,
    },
    CropTypeState {
        name: "oat",
        max_harvest: 5,
        min_harvest: 5,
        states: 5,
    },
    CropTypeState {
        name: "maize",
        max_harvest: 7,
        min_harvest: 7,
        states: 7,
    },
    CropTypeState {
        name: "sunflower",
        max_harvest: 8,
        min_harvest: 8,
        states: 8,
    },
    CropTypeState {
        name: "soybean",
        max_harvest: 7,
        min_harvest: 7,
        states: 7,
    },
    CropTypeState {
        name: "potato",
        max_harvest: 6,
        min_harvest: 6,
        states: 6,
    },
    CropTypeState {
        name: "sugarbeet",
        max_harvest: 8,
        min_harvest: 8,
        states: 8,
    },
    CropTypeState {
        name: "sugarcane",
        max_harvest: 8,
        min_harvest: 8,
        states: 8,
    },
    CropTypeState {
        name: "cotton",
        max_harvest: 9,
        min_harvest: 9,
        states: 9,
    },
    CropTypeState {
        name: "sorghum",
        max_harvest: 5,
        min_harvest: 5,
        states: 5,
    },
    CropTypeState {
        name: "grape",
        max_harvest: 11,
        min_harvest: 10,
        states: 7,
    },
    CropTypeState {
        name: "olive",
        max_harvest: 10,
        min_harvest: 9,
        states: 7,
    },
    CropTypeState {
        name: "poplar",
        max_harvest: 14,
        min_harvest: 14,
        states: 14,
    },
    CropTypeState {
        name: "grass",
        max_harvest: 4,
        min_harvest: 3,
        states: 4,
    },
    CropTypeState {
        name: "oilseedradish",
        max_harvest: 2,
        min_harvest: 2,
        states: 2,
    },
];

/// Base game weather definitions, FS22
pub const BG_CROP_WEATHER: [(&str, [CropSeason; 4]); 3] = [
    (
        "mapUS",
        [
            CropSeason {
                name: "spring",
                min: 6,
                max: 18,
            },
            CropSeason {
                name: "summer",
                min: 13,
                max: 34,
            },
            CropSeason {
                name: "autumn",
                min: 5,
                max: 25,
            },
            CropSeason {
                name: "winter",
                min: -11,
                max: 10,
            },
        ],
    ),
    (
        "mapFR",
        [
            CropSeason {
                name: "spring",
                min: 6,
                max: 18,
            },
            CropSeason {
                name: "summer",
                min: 13,
                max: 34,
            },
            CropSeason {
                name: "autumn",
                min: 5,
                max: 25,
            },
            CropSeason {
                name: "winter",
                min: -11,
                max: 10,
            },
        ],
    ),
    (
        "mapAlpine",
        [
            CropSeason {
                name: "spring",
                min: 5,
                max: 18,
            },
            CropSeason {
                name: "summer",
                min: 10,
                max: 30,
            },
            CropSeason {
                name: "autumn",
                min: 4,
                max: 22,
            },
            CropSeason {
                name: "winter",
                min: -12,
                max: 8,
            },
        ],
    ),
];

/// Base game crop definitions, FS22
pub const BG_CROPS: [Crop; 17] = [
    Crop {
        name: "wheat",
        growth_time: 8,
        harvest_periods: [
            false, false, false, false, true, true, false, false, false, false, false, false,
        ],
        plant_periods: [
            false, false, false, false, false, false, true, true, false, false, false, false,
        ],
    },
    Crop {
        name: "barley",
        growth_time: 7,
        harvest_periods: [
            false, false, false, true, true, false, false, false, false, false, false, false,
        ],
        plant_periods: [
            false, false, false, false, false, false, true, true, false, false, false, false,
        ],
    },
    Crop {
        name: "canola",
        growth_time: 9,
        harvest_periods: [
            false, false, false, false, true, true, false, false, false, false, false, false,
        ],
        plant_periods: [
            false, false, false, false, false, true, true, false, false, false, false, false,
        ],
    },
    Crop {
        name: "oat",
        growth_time: 5,
        harvest_periods: [
            false, false, false, false, true, true, false, false, false, false, false, false,
        ],
        plant_periods: [
            true, true, false, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "maize",
        growth_time: 7,
        harvest_periods: [
            false, false, false, false, false, false, false, true, true, false, false, false,
        ],
        plant_periods: [
            false, true, true, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "sunflower",
        growth_time: 8,
        harvest_periods: [
            false, false, false, false, false, false, false, true, true, false, false, false,
        ],
        plant_periods: [
            true, true, false, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "soybean",
        growth_time: 7,
        harvest_periods: [
            false, false, false, false, false, false, false, true, true, false, false, false,
        ],
        plant_periods: [
            false, true, true, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "potato",
        growth_time: 6,
        harvest_periods: [
            false, false, false, false, false, true, true, false, false, false, false, false,
        ],
        plant_periods: [
            true, true, false, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "sugarbeet",
        growth_time: 8,
        harvest_periods: [
            false, false, false, false, false, false, false, true, true, false, false, false,
        ],
        plant_periods: [
            true, true, false, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "sugarcane",
        growth_time: 8,
        harvest_periods: [
            false, false, false, false, false, false, false, true, true, false, false, false,
        ],
        plant_periods: [
            true, true, false, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "cotton",
        growth_time: 9,
        harvest_periods: [
            false, false, false, false, false, false, false, true, true, false, false, false,
        ],
        plant_periods: [
            true, false, false, false, false, false, false, false, false, false, false, true,
        ],
    },
    Crop {
        name: "sorghum",
        growth_time: 5,
        harvest_periods: [
            false, false, false, false, false, true, true, false, false, false, false, false,
        ],
        plant_periods: [
            false, true, true, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "grape",
        growth_time: 7,
        harvest_periods: [
            false, false, false, false, false, false, true, true, false, false, false, false,
        ],
        plant_periods: [
            true, true, true, false, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "olive",
        growth_time: 7,
        harvest_periods: [
            false, false, false, false, false, false, false, true, false, false, false, false,
        ],
        plant_periods: [
            true, true, true, true, false, false, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "poplar",
        growth_time: 14,
        harvest_periods: [
            true, true, true, true, true, true, true, true, true, true, true, true,
        ],
        plant_periods: [
            true, true, true, true, true, true, false, false, false, false, false, false,
        ],
    },
    Crop {
        name: "grass",
        growth_time: 4,
        harvest_periods: [
            true, true, true, true, true, true, true, true, true, true, true, true,
        ],
        plant_periods: [
            true, true, true, true, true, true, true, true, true, false, false, false,
        ],
    },
    Crop {
        name: "oilseedradish",
        growth_time: 2,
        harvest_periods: [
            true, true, true, true, true, true, true, true, true, true, true, true,
        ],
        plant_periods: [
            true, true, true, true, true, true, true, true, false, false, false, false,
        ],
    },
];
