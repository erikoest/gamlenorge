extern crate gamlenorge;

use gamlenorge::{Renderer, CONFIG};
use hoydedata::{set_map_dir, unmount_all_maps, Atlas, Result};

fn main() -> Result<()> {
    set_map_dir(&CONFIG.maps);

    let atlas1 = Atlas::new(1.0, None)?;
    let atlas10 = Atlas::new(10.0, None)?;

    let r = Renderer::new(atlas1, atlas10, None);
    let h = r?.find_horizon()?;
    println!("Horizon for target {}: {}", CONFIG.target, h);

    unmount_all_maps();

    Ok(())
}
