use std::fs;
use std::process::Command;

use crate::config::CONFIG;

use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref MOUNTLIST: ZipMountList = ZipMountList::new();
}

pub struct ZipMountList {
    mounts: Mutex<HashMap<String, bool>>,
}

impl ZipMountList {
    pub fn new() -> Self {
	Self {
	    mounts: Mutex::new(HashMap::new()),
	}
    }

    pub fn is_mounted(&self, filename: &str) -> bool {
	self.mounts.lock().unwrap().contains_key(filename)
    }
    
    pub fn register(&self, filename: &str) {
	self.mounts.lock().unwrap().insert(String::from(filename), true);
    }

    pub fn unmount_all(&self) {
	// Unmount all registered mount directories
	for k in self.mounts.lock().unwrap().keys() {
	    let absdir = format!("{}{}.dir", CONFIG.map_dir(), &k);

	    Command::new("/usr/bin/fusermount")
		.arg("-u")
		.arg(&absdir)
		.output()
		.expect("failed to execute process");

	    // Remove directory
	    fs::remove_dir(&absdir).unwrap();
	}

	self.mounts.lock().unwrap().clear();
    }
}

pub struct ZipMount {
    pub filename: String,
    pub directory: String,
}

impl ZipMount {
    pub fn new(filename: &str) -> Self {
	// Check that file is zip file
	if !(filename.ends_with(".zip")) {
	    panic!("No zip file");
	}

	let directory = format!("{}.dir/", filename);
	let absdir = format!("{}{}.dir", CONFIG.map_dir(), filename);

	if !MOUNTLIST.is_mounted(filename) {
	    // Only mount first time.
	    // Create directory <zipfile>.dir
	    fs::create_dir_all(&absdir).unwrap();

	    // fuse-zip -r <zipfile> <zipfile>.dir
	    Command::new("/usr/bin/fuse-zip")
		.arg("-r")
		.arg(format!("{}{}", CONFIG.map_dir(), &filename))
		.arg(absdir)
		.output()
		.expect("failed to execute process");

	    MOUNTLIST.register(filename);
	}

	Self {
	    filename: String::from(filename),
	    directory: directory.clone(),
	}
    }
}
