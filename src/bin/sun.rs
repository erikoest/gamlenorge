extern crate gamlenorge;

// Calculate position of the sun (azimuth and altitude) from geographic
// point and timestamp

use gamlenorge::{Result, Renderer, Coord};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let coord = Coord::from(String::from(&args[1]));
    let time = &args[2];

    let (az, alt) = Renderer::sun_position(&time, coord)?;

    println!("The position of the sun is {} / {}", az, alt);

    Ok(())
}
