use crate::errors::*;
use crate::map::Map;
use crate::coord::Coord;
use crate::config::CONFIG;
use crate::zipmount::ZipMount;
use crate::renderer::{OutputSender, Output};

use std::collections::HashMap;
use std::{fs, fmt};
use std::sync::Arc;

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::{ser::SerializeSeq, de::Visitor, de::SeqAccess};

/* 
For performance, maps are hashed up on low-res coordinates. Each low-res
coordinate points to a vector of candidate maps for the coordinate. Some
of these maps may be outside of the actual coordinate.
 */
pub struct Atlas {
    maps: HashMap<i32, Vec::<Arc<Map>>>,
}

impl Clone for Atlas {
    fn clone(&self) -> Self {
        Self {
            maps: self.maps.clone()
        }
    }
}

impl Atlas {
    pub fn new_from_directory(directory: &str, zipfile: &str,
                              tx: Option<&OutputSender>) -> Result<Self> {
	let absdir = format!("{}{}", CONFIG.map_dir(), directory);

	let mut maps = HashMap::new();

	for fentry in fs::read_dir(absdir)? {
            let path = fentry?.path();
            if path.is_dir() {
		continue;
            }

	    if !(path.extension().unwrap() == "tif") {
		continue;
	    }

	    let fname = path.file_name().unwrap().to_str().unwrap();
	    let dir_and_name = format!("{}{}", directory, fname);
	    let m = Arc::new(Map::new(&dir_and_name, &zipfile, tx)?);

	    for h in m.hashes() {
		if !maps.contains_key(&h) {
		    maps.insert(h, Vec::new());
		}
		maps.get_mut(&h).unwrap().push(Arc::clone(&m));
	    }
	}

	Ok(Self { maps: maps })
    }

    pub fn new_from_zip_file(file: &str, tx: Option<&OutputSender>)
                             -> Result<Self> {
	// Mount the zip file
	let zm = ZipMount::new(&file);

	Atlas::new_from_directory(&zm.directory, file, tx)
    }

    fn read_atlas(file: &str) -> Result<Self> {
        let data = fs::read_to_string(&file).expect("Unable to read file");
        let s = serde_json::from_str(&data)?;
	Ok(s)
    }

    pub fn write_atlas(&self, file: &str) -> Result<()> {
        let data = serde_json::to_string(&self)?;
        fs::write(&file, data).expect("Unable to write file");
	Ok(())
    }

    fn append(&mut self, other: &Atlas) {
	for (h, a) in other.maps.iter() {
	    if !self.maps.contains_key(&h) {
		self.maps.insert(*h, Vec::new());
	    }
	    for m in a.iter() {
		self.maps.get_mut(&h).unwrap().push(Arc::clone(&m));
	    }
	}
    }

    fn resolution(&self) -> f32 {
	self.maps.values().next().unwrap().into_iter().next().
	    unwrap().resolution()
    }
    
    fn new_empty() -> Self {
	Self {
	    maps: HashMap::new(),
	}
    }
    
    pub fn new(resolution: f32, tx: Option<&OutputSender>) -> Result<Self> {
	let mut s = Self::new_empty();
	let mut i = 0;

	// Read directory, for each atlas file, append it to our atlas
	for fentry in fs::read_dir(CONFIG.map_dir())? {
            let path = fentry?.path();
            if path.is_dir() {
		continue;
            }

	    // For some reason, !path.ends_with("atlas.json") does not work!
	    if !path.to_str().unwrap().ends_with("atlas.json") {
		continue;
	    }

	    let a = Self::read_atlas(path.to_str().unwrap())?;

	    // Check some map in atlas to determine if it has the right
	    // resolution.
	    if a.resolution() != resolution {
		continue;
	    }

	    s.append(&a);

	    i += 1;
	}

        if let Some(some_tx) = tx {
            some_tx.send(Output::Msg(format!(
	        "Read metadata for {} atlases with resolution {}.",
	        i, resolution))).unwrap();
        }

	Ok(s)
    }

    pub fn is_empty(&self) -> bool {
	self.maps.is_empty()
    }

    pub fn load_images(&self, coord: &Coord, tx: Option<&OutputSender>)
                       -> Result<()> {
	let h = Map::coord_to_hash(coord);

        if !self.maps.contains_key(&h) {
	    // No maps for coord. Do nothing.
            return Ok(());
	}

	for m in self.maps.get(&h).unwrap().iter() {
	    if !m.is_loaded() {
		m.load_image(tx)?;
	    }
	}

	Ok(())
    }

    pub fn has_maps(&self, coord: &Coord) -> bool {
	let h = Map::coord_to_hash(coord);

        self.maps.contains_key(&h)
    }
    
    pub fn has_images(&self, coord: &Coord) -> bool {
	let h = Map::coord_to_hash(coord);

        if !self.maps.contains_key(&h) {
	    // No maps available for coord
            return false;
	}

	for m in self.maps.get(&h).unwrap().iter() {
	    if !m.is_loaded() {
		return false;
	    }
	}

	true
    }
    
    pub fn lookup_maps(&self, coord: &Coord) -> Result<&Vec::<Arc<Map>>> {
	let h = Map::coord_to_hash(coord);
        self.maps.get(&h).ok_or("no map")?;
	Err(Error::MapNotFound(coord.clone()).into())
    }

    pub fn lookup(&self, coord: &Coord, tx: Option<&OutputSender>)
                  -> Result<f32> {
	/*
        Lookup function for coordinates. Find map which covers coordinates,
        lookup height.
	 */
        let h = Map::coord_to_hash(coord);

        if !self.maps.contains_key(&h) {
	    // No maps available for coord
            return Err(Error::MapNotFound(coord.clone()).into());
	}

	for m in self.maps.get(&h).unwrap().iter() {
            match m.lookup(coord) {
                Ok(r) => return Ok(r),
                Err(e) => {
                    if let Some(err) = e.downcast_ref::<Error>() {
                        if let Error::MapNotLoaded(_) = err {
                            // Load map and try again
                            m.load_image(tx)?;
                            if let Ok(r) = m.lookup(coord) {
                                return Ok(r)
                            }
                        }
                    }
                },
            }
	}

        Err(Error::MapNotFound(coord.clone()).into())
    }

    pub fn lookup_with_gradient(&self, coord: &Coord, tx: Option<&OutputSender>)
                                -> Result<(f32, f32, f32)> {
	/*
        Lookup function for coordinates. Find map which covers coordinates,
        lookup height.
	 */
        let h = Map::coord_to_hash(coord);

        if !self.maps.contains_key(&h) {
	    // No maps available for coord
            return Err(Error::MapNotFound(coord.clone()).into());
	}

	for m in self.maps.get(&h).unwrap().iter() {
            match m.lookup_with_gradient(coord) {
                Ok(r) => return Ok(r),
                Err(e) => {
                    if let Some(err) = e.downcast_ref::<Error>() {
                        if let Error::MapNotLoaded(_) = err {
                            // Load map and try again
                            m.load_image(tx)?;
                            if let Ok(r) = m.lookup_with_gradient(coord) {
                                return Ok(r)
                            }
                        }
                    }
                },
            }
	}

        Err(Error::MapNotFound(coord.clone()).into())
    }
}

impl Serialize for Atlas {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
	// Hash up maps on fname and serialize as a sequence
	let mut distinct = HashMap::new();

        for mv in self.maps.values() {
	    for m in mv.iter() {
		let name = m.fname.clone();
		if distinct.contains_key(&name) {
		    continue;
		}
		distinct.insert(name, m);
	    }
        }

	let mut seq = serializer.serialize_seq(Some(distinct.len()))?;
	// Serialize this with distinct.serialize()?
	for m in distinct.values() {
	    seq.serialize_element(m)?;
	}

        seq.end()
    }
}

struct VecMapDeserializer;

impl<'de> Visitor<'de> for VecMapDeserializer {
    type Value = Vec<Arc<Map>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("GeoTIFF map file.")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut v = Vec::new();

        while let Some(m) = seq.next_element::<Map>()? {
	    v.push(Arc::new(m));
        }

        Ok(v)
    }
}

impl<'de> Deserialize<'de> for Atlas {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
	let v = deserializer.deserialize_seq(VecMapDeserializer)?;

	let mut maps = HashMap::new();

	for m in v {
	    for h in m.hashes() {
		if !maps.contains_key(&h) {
		    maps.insert(h, Vec::new());
		}
		maps.get_mut(&h).unwrap().push(Arc::clone(&m));
	    }
	}

	Ok(Atlas { maps: maps })
    }
}
