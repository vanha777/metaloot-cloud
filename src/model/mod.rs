use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// Root struct
#[derive(Debug, Deserialize)]
pub struct Root {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: Vec<Item>,
}

// An item in the "Array"
#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: Resource,
}

// Resource structure
#[derive(Debug, Deserialize)]
pub struct Resource {
    pub fields: Vec<Field>,
    pub id: String,
}

// A field in the resource
#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    pub value: FieldValue,
}

// Value in a field
#[derive(Debug, Deserialize)]
pub struct FieldValue {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: JsonValue,
}

// Flattened output
#[derive(Debug, Serialize, Deserialize)]
pub struct FlattenedItem {
    pub uuid: String,
    pub id: String,
    pub attributes: HashMap<String, String>,
    pub name: String,
    pub description: String,
    pub thumbnail: String,
    pub metadata: HashMap<String, String>,
}

impl Resource {
    pub fn flatten(&self) -> FlattenedItem {
        let mut attributes = HashMap::new();
        let mut metadata = HashMap::new();
        let mut uuid = String::new();
        let mut id = String::new();
        let mut name = String::new();
        let mut description = String::new();
        let mut thumbnail = String::new();

        for field in &self.fields {
            match field.name.as_str() {
                "uuid" => {
                    if let Some(val) = field.value.value.as_str() {
                        uuid = val.to_string();
                    }
                }
                "id" => {
                    if let Some(val) = field.value.value.as_str() {
                        id = val.to_string();
                    }
                }
                "attributes" => {
                    if let JsonValue::Array(attr_array) = &field.value.value {
                        for entry in attr_array {
                            if let JsonValue::Object(map) = entry {
                                if let (Some(key), Some(val)) = (
                                    map.get("key")
                                        .and_then(|k| k.get("value").and_then(|v| v.as_str())),
                                    map.get("value")
                                        .and_then(|v| v.get("value").and_then(|v| v.as_str())),
                                ) {
                                    attributes.insert(key.to_string(), val.to_string());
                                }
                            }
                        }
                    }
                }
                "name" => {
                    if let Some(val) = field.value.value.as_str() {
                        name = val.to_string();
                    }
                }
                "description" => {
                    if let Some(val) = field.value.value.as_str() {
                        description = val.to_string();
                    }
                }
                "thumbnail" => {
                    if let Some(val) = field.value.value.as_str() {
                        thumbnail = val.to_string();
                    }
                }
                "metadata" => {
                    if let JsonValue::Array(meta_array) = &field.value.value {
                        for entry in meta_array {
                            if let JsonValue::Object(map) = entry {
                                if let (Some(key), Some(val)) = (
                                    map.get("key")
                                        .and_then(|k| k.get("value").and_then(|v| v.as_str())),
                                    map.get("value")
                                        .and_then(|v| v.get("value").and_then(|v| v.as_str())),
                                ) {
                                    metadata.insert(key.to_string(), val.to_string());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        FlattenedItem {
            uuid,
            id,
            attributes,
            name,
            description,
            thumbnail,
            metadata,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTItem {
    #[serde(rename = "itemName")]
    pub item_name: String,
    #[serde(rename = "itemType")] 
    pub item_type: String,
    pub attributes: NFTAttributes,
    #[serde(rename = "thumpNail")]
    pub thump_nail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTAttributes {
    pub key: String,
    pub value: String,
}
