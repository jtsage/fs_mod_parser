use std::{collections::HashMap, ops::{Deref, DerefMut}, sync::LazyLock};
use crate::files::PeriodMask;

use super::fruit_types::FruitTypes;
use super::growth::Growth;

/// Set of map crops
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct Crops(HashMap<String, Crop>);

impl Deref for Crops {
    type Target = HashMap<String, Crop>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Crops {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

/// Crop record
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Default)]
pub struct Crop {
    /// planting periods
    plant : PeriodMask,
    /// harvest periods
    harvest : PeriodMask,
}

/// Base game crops, FS22
static CROPS: LazyLock<Crops> = LazyLock::new(|| {
    Crops(HashMap::from([
        (String::from("wheat"),         Crop { plant: PeriodMask(0b0000_0011_0000), harvest: PeriodMask(0b0000_1100_0000) }),
        (String::from("barley"),        Crop { plant: PeriodMask(0b0000_0011_0000), harvest: PeriodMask(0b0001_1000_0000) }),
        (String::from("canola"),        Crop { plant: PeriodMask(0b0000_0110_0000), harvest: PeriodMask(0b0000_1100_0000) }),
        (String::from("oat"),           Crop { plant: PeriodMask(0b1100_0000_0000), harvest: PeriodMask(0b0000_1100_0000) }),
        (String::from("maize"),         Crop { plant: PeriodMask(0b0110_0000_0000), harvest: PeriodMask(0b0000_0001_1000) }),
        (String::from("sunflower"),     Crop { plant: PeriodMask(0b1100_0000_0000), harvest: PeriodMask(0b0000_0001_1000) }),
        (String::from("soybean"),       Crop { plant: PeriodMask(0b0110_0000_0000), harvest: PeriodMask(0b0000_0001_1000) }),
        (String::from("potato"),        Crop { plant: PeriodMask(0b1100_0000_0000), harvest: PeriodMask(0b0000_0110_0000) }),
        (String::from("sugarbeet"),     Crop { plant: PeriodMask(0b1100_0000_0000), harvest: PeriodMask(0b0000_0001_1000) }),
        (String::from("sugarcane"),     Crop { plant: PeriodMask(0b1100_0000_0000), harvest: PeriodMask(0b0000_0001_1000) }),
        (String::from("cotton"),        Crop { plant: PeriodMask(0b1000_0000_0001), harvest: PeriodMask(0b0000_0001_1000) }),
        (String::from("sorghum"),       Crop { plant: PeriodMask(0b0110_0000_0000), harvest: PeriodMask(0b0000_0110_0000) }),
        (String::from("grape"),         Crop { plant: PeriodMask(0b1100_0000_0000), harvest: PeriodMask(0b0000_0011_0000) }),
        (String::from("olive"),         Crop { plant: PeriodMask(0b1111_0000_0000), harvest: PeriodMask(0b0000_0001_0000) }),
        (String::from("poplar"),        Crop { plant: PeriodMask(0b1111_1100_0000), harvest: PeriodMask(0b1111_1111_1111) }),
        (String::from("grass"),         Crop { plant: PeriodMask(0b1111_1111_1000), harvest: PeriodMask(0b1111_1111_1111) }),
        (String::from("oilseedradish"), Crop { plant: PeriodMask(0b1111_1111_0000), harvest: PeriodMask(0b1111_1111_1111) }),
    ]))
});

impl Crops {
    /// Create crops from 2 file records
    #[must_use]
    pub fn from_files(fruits : &FruitTypes, growth : &Growth) -> Self {
        let mut crops = Self::default();
        for ( name, fruit ) in fruits.iter() {
            let harvest = growth.get_harvest(name, fruit);
            let plant = growth.get_planting(name);
            crops.insert(name.to_owned(), Crop { plant, harvest });
        }
        crops
    }

    /// Create crops from base only
    pub fn from_base() -> Self {
        CROPS.clone()
    }
}
