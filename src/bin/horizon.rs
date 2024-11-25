extern crate gamlenorge;

use gamlenorge::{Renderer, Result, Atlas, MOUNTLIST, CONFIG};

fn main() -> Result<()> {
    let atlas1 = Atlas::new(1.0, None)?;
    let atlas10 = Atlas::new(10.0, None)?;

    let r = Renderer::new(atlas1, atlas10, None);
    let h = r?.find_horizon()?;
    println!("Horizon for target {}: {}", CONFIG.target, h);
    MOUNTLIST.unmount_all();
    Ok(())
}
