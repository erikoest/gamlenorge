use lazy_static::lazy_static;
use std::sync::Mutex;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Progress {
    pub progress: Mutex<ProgressBar>,
}

lazy_static! {
    pub static ref PROGRESS: Progress = Progress::new();
}

impl Progress {
    pub fn new() -> Self {
	let p = ProgressBar::new(1);
	p.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{bar:40}] {pos:>7}/{len:7} lines")
		    .unwrap()
		    .progress_chars("=> "));
	
	Self {
	    progress: p.into(),
	}
    }

    pub fn set_length(&self, len: u64) {
	self.progress.lock().unwrap().set_length(len);
    }

    pub fn inc(&self, i: u64) {
	self.progress.lock().unwrap().inc(i);
    }

    // Print message above progress bar.
    pub fn println(&self, msg: &str) {
	let p = self.progress.lock().unwrap();

	if p.is_hidden() {
	    // If the bar is hidden, just print the message directly.
	    println!("{}", msg);
	}
	else {
	    // Let the bar print the message, putting it above the bar.
	    p.println(msg);
	}
    }
    
    pub fn finish(&self) {
	self.progress.lock().unwrap().finish();
    }

    pub fn hide(&self) {
	let mut p = self.progress.lock().unwrap();
	let h = ProgressBar::hidden();
	*p = h;
    }
}
