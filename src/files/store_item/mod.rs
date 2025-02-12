use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth};

use quick_xml::events::BytesStart;

/// vehicle types
mod vehicle;
/// placeable types
mod placeable;

use vehicle::Vehicle;
use placeable::Placeable;

/// Store item definition
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, Default)]
pub struct StoreItem {
    /// item type
    #[serde(rename="itemType")]
    item_type : StoreItemType,
    /// vehicle record
    #[serde(skip_serializing_if = "Option::is_none")]
    vehicle : Option<Vehicle>,
    /// placable record
    #[serde(skip_serializing_if = "Option::is_none")]
    placeable : Option<Placeable>,
}

/// Known store item types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="lowercase")]
enum StoreItemType {
    /// trap for unknown
    #[default]
    Unknown,
    /// vehicles and implements
    Vehicle,
    /// placables
    Placeable,
}

///Capability
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[serde(from="bool", into="bool")]
pub enum Capability {
    /// Has option
    Yes,
    /// Does not have option
    #[default]
    No,
}

impl From<Capability> for bool {
    fn from(value: Capability) -> Self { matches!(value, Capability::Yes) }
}
impl From<bool> for Capability {
    fn from(value: bool) -> Self { if value { Self::Yes } else { Self::No } }
}

impl XMLReader<Self> for StoreItem {
    /// Load the item from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut item = Self::default().read_xml(xml_text).cloned()?;

        match item.item_type {
            StoreItemType::Unknown => Err(AbstractFileError::XmlWrongFileType),
            StoreItemType::Vehicle => {
                item.vehicle = Some(Vehicle::from_string(xml_text)?);
                Ok(item)
            },
            StoreItemType::Placeable => {
                item.placeable = Some(Placeable::from_string(xml_text)?);
                Ok(item)
            },
        }
    }

    #[inline]
    fn tags_paired(&mut self, e: &BytesStart, depth : i32, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"vehicle", 0) => {
                self.item_type = StoreItemType::Vehicle;
                Self::slurp(e, reader)
            },
            (b"placeable", 0) => {
                self.item_type = StoreItemType::Placeable;
                Self::slurp(e, reader)
            },
            (_, 0) => Self::slurp(e, reader),
            _ => Ok(1),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neither_type() {
        let xml = r#"<?xml version=\"1.0\" ?><poop type="trailer"></poop>"#;
        let actual = StoreItem::from_string(xml);

        assert_eq!(actual, Err(AbstractFileError::XmlWrongFileType));
    }
}
