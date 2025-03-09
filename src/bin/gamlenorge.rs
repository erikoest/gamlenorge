extern crate gamlenorge;

use gamlenorge::{Renderer, CONFIG};
use hoydedata::{set_map_dir, unmount_all_maps, Result};

fn main() -> Result<()> {
    set_map_dir(&CONFIG.map_dir());

    let _ = Renderer::render()?;

    unmount_all_maps();

    Ok(())
}
