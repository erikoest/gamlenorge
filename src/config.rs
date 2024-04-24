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
    pub min_depth: f32,
    pub max_depth: f32,
    pub observer_height_offset: f32,
    pub target_height_offset: f32,
    pub green_limit: f32,
    pub water_level: f32,
    pub haziness: f32,
    pub sky_lum: f32,
    pub rayleigh: f32,
    pub water_shinyness: f32,
    pub water_ripples: f32,
    pub water_reflection_iterations: u16,
    pub time: String,
    pub output: String,
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
		("observer_height_offset", "10"),
		("target_height_offset", "10"),
		("width", "1600"),
		("height", "200"),
		("width_angle", "0.6"),
		("min_depth", "0"),
		("max_depth", "150000"),
		("haziness", "0.7"),
		("green_limit", "800"),
		("water_level", "0"),
		("sky_lum", "1"),
		("rayleigh", "1"),
		("water_shinyness", "0.5"),
		("water_ripples", "1"),
		("water_reflection_iterations", "10"),
		("time", "2023-07-01T18:00:00+0200"),
		("output", "out.tif"),
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
