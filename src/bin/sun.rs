extern crate gamlenorge;

// Calculate position of the sun (azimuth and altitude) from geographic
// point and timestamp

use gamlenorge::Renderer;
use hoydedata::{Result, Coord};

use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
	println!("Usage: sun <coordinate> <time>");
	return Ok(());
    }

    let coord = Coord::from(args[1].as_str());
    let time = &args[2];

    let (az, alt) = Renderer::sun_position(&time, coord)?;

    println!("The position of the sun is {} / {}",
	     az.to_degrees(), alt.to_degrees());

    Ok(())
}
