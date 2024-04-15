extern crate gamlenorge;

use gamlenorge::{Atlas, Renderer, Result, MOUNTLIST, CONFIG};

fn main() -> Result<()> {
    let a1 = Atlas::new(1.0)?;
    let a10 = Atlas::new(10.0)?;

    let r = Renderer::new(a1, a10);
    let h = r?.find_horizon()?;
    println!("Horizon for target {}: {}", CONFIG.target, h);
    MOUNTLIST.unmount_all();
    Ok(())
}
