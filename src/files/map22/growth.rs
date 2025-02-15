use std::{ops::{DerefMut, Deref}, collections::HashMap};
use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth, PeriodMask};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::fruit_types::FruitType;

/// New growth state from current growth state
#[derive(Debug, Default, Clone, Copy)]
pub struct GrowNewState([usize; 20]);

impl Deref for GrowNewState {
    type Target = [usize; 20];

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for GrowNewState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Growth for a crop
#[derive(Debug, Default, Clone, Copy)]
pub struct GrowCrop {
    /// new states
    new_states : GrowNewState,
    /// can plant here
    plant : bool,
}

/// 12 months of [`GrowCrop`]
#[derive(Debug, Default, Clone, Copy)]
pub struct GrowCrops([GrowCrop; 12]);

impl Deref for GrowCrops {
    type Target = [GrowCrop; 12];

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for GrowCrops {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Growth for a named crop
#[derive(Debug, Default, Clone)]
pub struct Growth(HashMap<String, GrowCrops>);

impl Deref for Growth {
    type Target = HashMap<String, GrowCrops>;

    fn deref(&self) -> &Self::Target { &self.0 }
}


impl XMLReader<Self> for Growth {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        Self::default().read_xml(xml_text).cloned()
    }

    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"growth", 0) => { Ok(1) },
            (b"fruit", 2) => { self.tag_fruit(reader, e); Ok(0) },
            (_, 0)        => Err(AbstractFileError::XmlWrongFileType),
            _ => Ok(1),
        }
    }
}

impl Growth {
    /// Do season
    #[inline]
    fn tag_fruit(&mut self, reader: &mut Reader<&[u8]>, e : &BytesStart) {
        let mut buf_next = Vec::new();
        let mut seasonal = false;

        let Some(fruit_name) = Self::xml_attribute(e, "name") else { return };

        let mut fruit = GrowCrops::default();

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Start(e)) if e.name().as_ref() == b"period" => {
                    seasonal = true;
                    let Some(period_index) = Self::xml_attribute_number(&e, "index").map(|v:usize| v - 1) else { continue };
                    if let Some(plant) = Self::xml_attribute(&e, "plantingAllowed") {
                        if plant == *"true" { fruit[period_index].plant = true; }
                    }
                    fruit[period_index].new_states = Self::tag_period(reader, &e);
                }
                Ok(Event::Empty(e)) if e.name().as_ref() == b"period" => {
                    seasonal = true;
                    let Some(period_index) = Self::xml_attribute_number(&e, "index").map(|v:usize| v - 1) else { continue };
                    if let Some(plant) = Self::xml_attribute(&e, "plantingAllowed") {
                        if plant == *"true" { fruit[period_index].plant = true; }
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }

        if seasonal && fruit_name.to_ascii_lowercase() != *"meadow" {
            self.0.insert(fruit_name.to_ascii_lowercase(), fruit);
        }
    }

    /// Process period
    #[inline]
    fn tag_period(reader: &mut Reader<&[u8]>, e : &BytesStart) -> GrowNewState {
        let mut buf_next = Vec::new();

        let mut new_state = GrowNewState::default();

        loop {
            match reader.read_event_into(&mut buf_next) {
                Ok(Event::Empty(e)) if e.name().as_ref() == b"update" => {
                    let Some(range_str) = Self::xml_attribute(&e, "range") else { continue };

                    if let Some(range) = if range_str.contains('-') {
                        let x:Vec<usize> = range_str.split('-').map(|v| v.parse::<usize>().unwrap_or_default()).collect();
                        (x.len() == 2).then(|| x[0]..=x[1])
                    } else if let Ok(range_i) = range_str.parse::<usize>() {
                        Some(range_i..=range_i)
                    } else {
                        None
                    } {
                        if let Some(add) = Self::xml_attribute_number::<usize>(&e, "add") {
                            for input in range {
                                new_state[input-1] = input + add;
                            }
                        } else if let Some(set) = Self::xml_attribute_number::<usize>(&e, "set") {
                            for input in range {
                                new_state[input-1] = set;
                            }
                        }
                    }
                }
                Ok(Event::End(f)) if f.name() == e.name() => break,
                _ => (),
            }
        }

        new_state
    }

    /// Get planting months
    pub fn get_planting<S: AsRef<str>>(&self, name : S) -> PeriodMask {
        self.get(name.as_ref()).map_or_else(PeriodMask::default, |crop| PeriodMask::from(crop
            .iter().enumerate()
            .filter(|(_, v)| v.plant )
            .map(|(i, _)| (i + 1) )
            .collect::<Vec<usize>>()
        ))
    }

    /// Get harvest months from plant period start
    fn single_harvest<S: AsRef<str>>(&self, name : S, start_period : usize, min : usize, max : usize) -> PeriodMask {
        let grow_time = if min > 12 { 24 } else { 12 };
        let mut can_harvest = PeriodMask::default();

        if let Some(crop) = self.get(name.as_ref()) {
            let mut current_state = 1_usize;
            
            for p in start_period..=start_period+grow_time {
                let p_index = (p - 1) % 12;
                let p_ready = (p % 12) + 1;
                let new_state = crop[p_index].new_states[current_state - 1];
                if new_state != 0 { current_state = new_state; }
                if current_state >= min && current_state <= max {
                    can_harvest.set(p_ready);
                }
            }
        }
        can_harvest
    }

    /// Get harvest periods from min and max
    pub fn get_harvest<S: AsRef<str>>(&self, name : S, fruit : &FruitType) -> PeriodMask {
        let mut can_harvest = PeriodMask::default();
        let plant_periods:Vec<usize> = self.get_planting(&name).into();

        for period in plant_periods {
            can_harvest |= self.single_harvest(&name, period, fruit.min_harvest, fruit.max_harvest);
        }

        can_harvest
    }
}


// MARK: TESTING
#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::AbstractFile;
    use pretty_assertions::assert_eq;

    #[test]
    fn wrong_type() {
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?><barf></barf>"#;
        let actual = Growth::from_string(xml);
        assert_eq!(actual.unwrap_err(), AbstractFileError::XmlWrongFileType);
    }

    #[test]
    fn from_file() {
        let filename = "tests/test_mods/MAP_AddedCrops.zip";
        let item = "maps/xml/maps_growth.xml";
        let dump = false;

        let mut file_handle = AbstractFile::new(filename);
        let actual = Growth::from_abstract_file(&mut file_handle, item).unwrap();

        if dump { println!("{:?}", actual); }

        let known_crops = [
            ("wheat", vec![7,8], vec![5,6], FruitType::singleton(8)),
            ("barley", vec![7,8], vec![4,5], FruitType::singleton(7)),
            ("canola", vec![6,7], vec![5,6], FruitType::singleton(9)),
            ("oat", vec![1,2], vec![5,6], FruitType::singleton(5)),
            ("maize", vec![2,3], vec![8,9], FruitType::singleton(7)),
            ("sunflower", vec![1,2], vec![8,9], FruitType::singleton(8)),
            ("soybean", vec![2,3], vec![8,9], FruitType::singleton(7)),
            ("potato", vec![1,2], vec![6,7], FruitType::singleton(6)),
            ("sugarbeet", vec![1,2], vec![8,9], FruitType::singleton(8)),
            ("sugarcane", vec![1,2], vec![8,9], FruitType::singleton(8)),
            ("cotton", vec![1,12], vec![8,9], FruitType::singleton(9)),
            ("sorghum", vec![2,3], vec![6,7], FruitType::singleton(5)),
            ("grape", vec![1,2,3], vec![7,8], FruitType { max_harvest : 11, min_harvest : 10 }),
            ("olive", vec![1,2,3,4], vec![8], FruitType { max_harvest : 10, min_harvest : 9 }),
            ("poplar", vec![1,2,3,4,5,6], vec![1,2,3,4,5,6,7,8,9,10,11,12], FruitType::singleton(14)),
            ("grass", vec![1,2,3,4,5,6,7,8,9], vec![1,2,3,4,5,6,7,8,9,10,11,12], FruitType { max_harvest : 4, min_harvest : 3 }),
            ("oilseedradish", vec![1,2,3,4,5,6,7,8], vec![1,2,3,4,5,6,7,8,9,10,11,12], FruitType::singleton(2)),
            ("clover", vec![1,2,3,4,5,6,7,8,9], vec![2,3,4,5,6,7,8,9], FruitType { max_harvest: 7, min_harvest: 5 }),
            ("alfalfa", vec![1,2,3,4,5,6,7,8,9], vec![2,3,4,5,6,7,8,9], FruitType { max_harvest: 7, min_harvest: 5 }),
            ("silage_corn", vec![2,3], vec![8,9,10,11], FruitType::singleton(7))
        ];

        for (name, plant, harvest, fruit) in known_crops {
            let plant_actual = actual.get_planting(name);
            let harvest_actual = actual.get_harvest(name, &fruit);
            assert_eq!(PeriodMask::from(plant), plant_actual, "{name} planting");
            assert_eq!(PeriodMask::from(harvest), harvest_actual, "{name} harvesting");
        }
    }
}
