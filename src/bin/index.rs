extern crate gamlenorge;

use gamlenorge::{Atlas, Result, MOUNTLIST, CONFIG};
use std::env;

fn main() -> Result<()> {
    let dir = &CONFIG.map_dir();
    let args: Vec<String> = env::args().collect();
    let file = &args[1];
    let afile;
    let a;

    if file == "" {
	// No file. Index directory.
	a = Atlas::new_from_directory("", "")?;
	afile = format!("{}{}", dir, "atlas.json");
    }
    else {
	a = Atlas::new_from_zip_file(&file)?;
	afile = format!("{}{}{}", CONFIG.map_dir(), file, ".atlas.json");
    }

    a.write_atlas(&afile)?;

    MOUNTLIST.unmount_all();

    Ok(())
}
