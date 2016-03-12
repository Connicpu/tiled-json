use std::str;
use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs::File;
use std::collections::HashMap;

use {GlobalTile, LocalTile};

use serde::{Deserialize, Deserializer};

use serde_json;
use serde_json::Value as JsonValue;
use serde_json::Error as JsonError;

/// Tiled Tileset, containing everything we need to render tiles from
/// this set as well as decide how to do collision checks
#[derive(Clone, Debug, Deserialize)]
pub struct Tileset {
    /// Name of the tileset specified by its creator
    pub name: String,
    /// Global ID of the first tile which is part of this set. Global IDs
    /// are meaningless unless applied to a list of Tilesets associated with
    /// the correct map.
    pub firstgid: GlobalTile,
    
    /// Number of tiles contained in this map
    pub tilecount: u32,
    /// Height in pixels of each tile
    pub tileheight: u32,
    /// Width in pixels of each tile
    pub tilewidth: u32,
    
    /// The number of tiles per row in the image
    pub columns: u32,
    /// Path to the image representing this tileset
    /// TODO: Support multi-image sets?
    pub image: PathBuf,
    /// Expected height in pixels of the image
    pub imageheight: u32,
    /// Expected width in pixels of the image
    pub imagewidth: u32,
    /// Margin in the image between the edges and where the first tile starts
    pub margin: u32,
    /// Number of pixels between each tile
    pub spacing: u32,
    
    /// Key-Value pair properties specified for this tileset (game-specific data)
    pub properties: Option<HashMap<String, String>>,
    /// List of all the terrain types defined in this tileset. The values inside
    /// the `tiles` member correspond to indices in this array
    pub terrains: Vec<Terrain>,
    /// Key-Value pair properties associated with specific tiles in this set
    pub tileproperties: TileProperties,
    /// List of tiles that are associated with specific terrain, and which
    /// corners belong to which terrain type.
    pub tiles: TileTerrain,
}

impl Tileset {
    /// Given a JsonValue for a tileset, and the path of the level it is a member of,
    /// try to parse the tileset or load and parse it from an external file.
    pub fn load<P: AsRef<OsStr>>(data: JsonValue, data_path: &P) -> Result<Tileset, JsonError> {
        use serde::de::Error;
        // The data we're deserializing here must be a Json table
        let mut data = match data {
            JsonValue::Object(data) => data,
            _ => return Err(JsonError::custom("Tileset data was not an Object")),
        };
        
        // If data contains a "source" field, we're dealing with an
        // external tileset, and we must load that file.
        Ok(match data.remove("source") {
            Some(JsonValue::String(source)) => {
                // firstgid is not stored in the external data, so we
                // must save it from here for later
                let firstgid = match data.remove("firstgid").and_then(|i| i.as_u64()) {
                    Some(i) => i as u32,
                    None => return Err(JsonError::custom("Tileset had no firstgid")),
                };
                
                // Start with the path to the level
                let mut path = PathBuf::from(data_path);
                path.pop(); // Path is now the level directory
                path.push(source); // Path is the tileset to load
                
                // Try to open the file! We can just use the try!() macro
                // because serde_json::Error has a From converion from io::Error
                let mut file = try!(File::open(&path));
                
                // Parse the tileset file into an ExternalTileset structure
                let ext: ExternalTileset = try!(serde_json::from_reader(&mut file));
                
                path.pop();
                path.push(&ext.image);
                
                Tileset {
                    name: ext.name,
                    firstgid: GlobalTile(firstgid),
                    
                    tilecount: ext.tilecount,
                    tileheight: ext.tileheight,
                    tilewidth: ext.tilewidth,
                    
                    columns: ext.columns,
                    image: path,
                    imageheight: ext.imageheight,
                    imagewidth: ext.imagewidth,
                    margin: ext.margin,
                    spacing: ext.spacing,
                    
                    properties: ext.properties,
                    terrains: ext.terrains,
                    tileproperties: ext.tileproperties,
                    tiles: ext.tiles,
                }
            },
            // The tileset is inlined in the level, just parse its data
            _ => {
                let mut tileset: Tileset = try!(serde_json::from_value(JsonValue::Object(data)));
                let mut path = PathBuf::from(data_path);
                path.pop();
                path.push(&tileset.image);
                tileset.image = path;
                tileset
            }
        })
    }
    
    pub fn contains_tile(&self, id: GlobalTile) -> bool {
        if id.0 < self.firstgid.0 { return false; }
        let local = id.0 - self.firstgid.0;
        local < self.tilecount
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ExternalTileset {
    name: String,
    
    tilecount: u32,
    tileheight: u32,
    tilewidth: u32,
    
    columns: u32,
    image: PathBuf,
    imageheight: u32,
    imagewidth: u32,
    margin: u32,
    spacing: u32,
    
    properties: Option<HashMap<String, String>>,
    terrains: Vec<Terrain>,
    tileproperties: TileProperties,
    tiles: TileTerrain,
}

#[derive(Clone, Debug)]
pub struct TileProperties {
    pub tiles: HashMap<LocalTile, HashMap<String, String>>,
}

impl Deserialize for TileProperties {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        // Tiled uses string keys because it's a sparse array,
        // so we're just going to parse it like that and then
        // convert them to LocalTiles
        let data: HashMap<String, HashMap<String, String>>;
        data = try!(Deserialize::deserialize(d));
        
        let mut props = HashMap::new();
        for (k, v) in data {
            // Allows us to return an error when a bad key is present
            use serde::de::Error;
            
            // We'll return an error if the key isn't a valid integer
            let id: u32 = match str::parse(&k) {
                Ok(id) => id,
                Err(_) => return Err(D::Error::custom("tileproperties contained a non-integer key"))
            };
            
            props.insert(LocalTile(id), v);
        }
        
        Ok(TileProperties {
            tiles: props,
        })
    }
}

#[derive(Clone, Debug)]
pub struct TileTerrain {
    pub tiles: HashMap<LocalTile, [u32; 4]>
}

impl Deserialize for TileTerrain {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Data {
            terrain: [u32; 4]
        }
        
        // Tiled uses string keys because it's a sparse array,
        // so we're just going to parse it like that and then
        // convert them to LocalTiles
        let data: HashMap<String, Data>;
        data = try!(Deserialize::deserialize(d));
        
        let mut terrains = HashMap::new();
        for (k, v) in data {
            // Allows us to return an error when a bad key is present
            use serde::de::Error;
            
            // We'll return an error if the key isn't a valid integer
            let id: u32 = match str::parse(&k) {
                Ok(id) => id,
                Err(_) => return Err(D::Error::custom("tileproperties contained a non-integer key"))
            };
            
            terrains.insert(LocalTile(id), v.terrain);
        }
        
        Ok(TileTerrain {
            tiles: terrains,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Terrain {
    pub name: String,
    pub tile: LocalTile,
}

/// Test to ensure we can deserialize an ExternalTileset
#[test]
fn deserialize_external() {
    use serde_json::from_str;
    
    let data = include_str!("../test-assets/tilesets/goodly-2x.json");
    let _: ExternalTileset = from_str(data).unwrap();
}
