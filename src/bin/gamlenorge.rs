extern crate gamlenorge;

use gamlenorge::{Renderer, Result, MOUNTLIST};

fn main() -> Result<()> {
    let _ = Renderer::render()?;
    MOUNTLIST.unmount_all();
    Ok(())
}
