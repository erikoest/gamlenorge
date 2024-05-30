extern crate gamlenorge;

use gamlenorge::{Atlas, Renderer, Result, MOUNTLIST};

fn main() -> Result<()> {
    let a1 = Atlas::new(1.0)?;
    let a10 = Atlas::new(10.0)?;

    let mut r = Renderer::new(a1, a10)?;
    let _ = r.render()?;
    MOUNTLIST.unmount_all();
    Ok(())
}
