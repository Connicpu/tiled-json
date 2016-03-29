use std::path::Path;
use std::fs::File;
use std::collections::HashMap;
use layer::Layer;
use tileset::Tileset;
use serde_json;
use serde_json::Value as JsonValue;
use serde_json::Error as JsonError;

#[derive(Clone, Debug)]
pub struct Level {
    pub height: u32,
    pub width: u32,
    
    pub properties: HashMap<String, String>,
    
    pub orientation: String,
    pub renderorder: String,
    
    pub tileheight: u32,
    pub tilewidth: u32,
    
    pub layers: Vec<Layer>,
    pub tilesets: Vec<Tileset>,
}

impl Level {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Level, JsonError> {
        let mut file = try!(File::open(&path));
        let level: IntermediateLevel = try!(serde_json::from_reader(&mut file));
        
        let tilesets: Vec<Tileset> = try!(level.tilesets.into_iter().map(|data| {
            Tileset::load(data, &path.as_ref())
        }).collect());
        
        Ok(Level {
            height: level.height,
            width: level.width,
            
            properties: level.properties,
            
            orientation: level.orientation,
            renderorder: level.renderorder,
            
            tileheight: level.tileheight,
            tilewidth: level.tilewidth,
            
            layers: level.layers,
            tilesets: tilesets,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct IntermediateLevel {
    height: u32,
    width: u32,
    
    properties: HashMap<String, String>,
    
    orientation: String,
    renderorder: String,
    
    tileheight: u32,
    tilewidth: u32,
    
    layers: Vec<Layer>,
    tilesets: Vec<JsonValue>,
}

#[test]
pub fn load_level() {
    let path = "test-assets/levels/simple2.json";
    let _ = Level::load(path).unwrap();
}

