use indicatif::{ProgressBar, ProgressStyle};

pub struct Progress {
    pub progress: ProgressBar,
}

impl Progress {
    pub fn new() -> Self {
	let p = ProgressBar::new(1);
	p.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{bar:40}] {pos:>7}/{len:7} lines")
		    .unwrap()
		    .progress_chars("=> "));
	
	Self {
	    progress: p,
	}
    }

    pub fn set_length(&self, len: u64) {
	self.progress.set_length(len);
    }

    pub fn inc(&self, i: u64) {
	self.progress.inc(i);
    }

    // Print message above progress bar.
    pub fn println(&self, msg: &str) {
	self.progress.println(msg);
    }
    
    pub fn finish(&self) {
	self.progress.finish();
    }
}
