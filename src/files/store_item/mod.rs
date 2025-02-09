use crate::errors::AbstractFileError;
use crate::files::{XMLReader, XMLReaderDepth};

use quick_xml::events::BytesStart;

/// vehicle types
mod vehicle;

use vehicle::Vehicle;

/// Store item definition
#[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, Default)]
struct StoreItem {
    /// item type
    #[serde(rename="itemType")]
    item_type : StoreItemType,
    /// vehicle record
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    vehicle : Option<Vehicle>,
    /// placable record
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    placeable : Option<bool>,
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

impl XMLReader<Self> for StoreItem {
    /// Load the modDesc.xml from an already decoded string
    fn from_string(xml_text: &str) -> Result<Self, AbstractFileError> {
        let mut item = Self::default().read_xml(xml_text).cloned()?;

        match item.item_type {
            StoreItemType::Unknown => Err(AbstractFileError::XmlWrongFileType),
            StoreItemType::Vehicle => {
                item.vehicle = Some(Vehicle::from_string(xml_text)?);
                Ok(item)
            },
            StoreItemType::Placeable => {
                println!("PLACE");
                Ok(item)

            },
        }
    }

    #[inline]
    fn tags_paired(e: &BytesStart, depth : i32, data: &mut Self, reader: &mut quick_xml::Reader<&[u8]>) -> XMLReaderDepth {
        match (e.name().as_ref(), depth) {
            (b"vehicle", 0) => {
                data.item_type = StoreItemType::Vehicle;
                Self::slurp(e, reader)
            },
            (b"placeable", 0) => {
                data.item_type = StoreItemType::Placeable;
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
