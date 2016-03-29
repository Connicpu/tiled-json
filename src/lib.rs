#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Deserializer};

pub mod layer;
pub mod level;
pub mod tileset;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GlobalTile(pub u32);

impl GlobalTile {
    /// From this GlobalTile, given the set of tilesets associated with the
    /// map, find the Tileset and LocalTile this ID belongs to, or None
    /// if it does not belong to any.
    pub fn find_local(self, sets: &[tileset::Tileset]) -> Option<(usize, LocalTile)> {
        for (i, set) in sets.iter().enumerate() {
            if set.contains_tile(self) {
                let id = LocalTile(self.0 - set.firstgid.0);
                return Some((i, id))
            }
        }
        None
    }
}

impl Deserialize for GlobalTile {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        // These are just wrapper structs, the values
        // should be decoded as a plain u32
        Ok(GlobalTile(try!(u32::deserialize(d))))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LocalTile(pub u32);

impl Deserialize for LocalTile {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
        // These are just wrapper structs, the values
        // should be decoded as a plain u32
        Ok(LocalTile(try!(u32::deserialize(d))))
    }
}
