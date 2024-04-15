use crate::coord::Coord;

use serde::{Deserialize, Serialize};
use config::{*, ext::*};
use std::env;
use lazy_static::lazy_static;
use configparser::ini::Ini;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub maps: String,
    pub observer: Coord,
    pub target: Coord,
    pub width: u32,
    pub height: u32,
    pub width_angle: f32,
    pub max_depth: f32,
    pub observer_height_offset: f32,
    pub target_height_offset: f32,
    pub haziness: f32,
    pub sun_height_angle: f32,
    pub sun_compass_angle: f32,
    pub output: String,
    pub skip_distance: f32,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

impl Config {
    pub fn new() -> Self {
	let mut cfname = String::from("gamlenorge.ini");

	// Fish out --config from command line
	let mut cnext = false;
	for a in env::args() {
	    if cnext {
		cfname = a;
		cnext = false;
		continue;
	    }
	
	    if a == "-c" || a == "--config" {
		cnext = true;
	    }
	}

	/* Read config from ini-file. Note: We cannot use the add_ini_file
	here since it would load parameters under the default section (and
	other sections). We want the parameters to be put on the root level.
	As a workaround, we put the arguments into a vector and add it as 
	an extra source of command line arguments.
	*/
	let mut iniparser = Ini::new();
	let ini = iniparser.load(cfname).unwrap();
	let mut ini_vec = Vec::new();
	for (k, v) in ini["default"].clone() {
	    ini_vec.push(format!("--{}", k));
	    ini_vec.push(v.unwrap());
	}

	let ini_src = CommandLineConfigurationSource::from(ini_vec.iter());
	let mut builder = DefaultConfigurationBuilder::new();
            builder.add_in_memory(&[
		("maps", "/media/ekstern/hoydedata"),
		("observer", "Nordre Trolltind"),
		("target", "Store Vengetind"),
		("width", "1600"),
		("height", "200"),
		("width_angle", "0.6"),
		("max_depth", "150000"),
		("observer_height_offset", "10"),
		("target_height_offset", "-200"),
		("haziness", "0.7"),
		("sun_height_angle", "10"),
		("sun_compass_angle", "270"),
		("output", "out.tif"),
		("skip_distance", "0"),
	    ]);
	builder.add(Box::new(ini_src));
	// builder.add_env_vars();
	let config = builder.add_command_line()
            .build()
            .unwrap();

	config.reify()
    }

    pub fn map_dir(&self) -> String {
	let mut md = self.maps.clone();
	if !md.ends_with("/") {
	    md.push('/');
	}

	md
    }
}
