use std::collections::HashMap;
use GlobalTile;
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use serde_json::from_value;

#[derive(Clone, Debug)]
pub enum Layer {
    Tiles(TileLayer),
    Objects(ObjectLayer),
}

impl Layer {
    pub fn name(&self) -> &str {
        match *self {
            Layer::Tiles(ref tiles) => &tiles.name,
            Layer::Objects(ref objects) => &objects.name,
        }
    }
}

impl Deserialize for Layer {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        use serde::de::Error as SerdeError;
        use std::error::Error;
        let data = try!(JsonValue::deserialize(d));
        let kind = match data {
            JsonValue::Object(ref data) => match data.get("type") {
                Some(&JsonValue::String(ref kind)) => kind.clone(),
                _ => return Err(D::Error::custom("Layer does not have `type` field")),
            },
            _ => return Err(D::Error::custom("Layer was not a table")),
        };
        
        Ok(match &kind[..] {
            "tilelayer" => Layer::Tiles(match from_value(data) {
                Ok(layer) => layer,
                Err(e) => return Err(D::Error::custom(
                    Into::<String>::into("tilelayer failed ") + e.description()
                )),
            }),
            "objectgroup" => Layer::Objects(match from_value(data) {
                Ok(layer) => layer,
                Err(e) => return Err(D::Error::custom(
                    Into::<String>::into("objectgroup failed ") + e.description()
                )),
            }),
            _ => return Err(D::Error::custom("Unknown layer type")),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TileLayer {
    pub name: String,
    pub opacity: f32,
    pub properties: Option<HashMap<String, String>>,
    pub visible: bool,
    pub width: u32,
    pub height: u32,
    pub x: f32,
    pub y: f32,
    
    pub data: Vec<GlobalTile>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ObjectLayer {
    pub name: String,
    pub opacity: f32,
    pub properties: Option<HashMap<String, String>>,
    pub visible: bool,
    pub width: u32,
    pub height: u32,
    pub x: f32,
    pub y: f32,
    
    pub draworder: String,
    pub objects: Vec<Object>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Object {
    pub id: u32,
    pub name: String,
    
    #[serde(rename = "type")]
    pub _type: String,
    pub gid: Option<GlobalTile>,
    pub ellipse: Option<bool>,
    pub polygon: Option<Vec<PolyPoint>>,
    
    pub properties: Option<HashMap<String, String>>,
    pub rotation: f32,
    pub visible: bool,
    
    pub height: f32,
    pub width: f32,
    
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct PolyPoint {
    pub x: f32,
    pub y: f32,
}
