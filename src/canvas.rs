use crate::config::CONFIG;
use crate::color::Color;
use image::Rgb;
use sdl2::video::Window;
use sdl2::rect::Point;

use std::io;
use std::io::prelude::*;

pub struct Canvas {
    im: image::ImageBuffer<Rgb<u8>, Vec<u8>>,
    canvas: Option<sdl2::render::Canvas<Window>>,
    last_y: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
	let im = image::ImageBuffer::new(width, height);

	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
    
	let window = video_subsystem.window(
	    &CONFIG.output, width, height).build().unwrap();

	let optc : Option<sdl2::render::Canvas<Window>>;

	if CONFIG.headless {
	    optc = None;
	}
	else {
	    let mut c = window.into_canvas()
		.present_vsync()
		.build().unwrap();

	    c.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
	    c.clear();
	    
	    optc = Some(c);
	}

	Self {
	    im: im,
	    canvas: optc,
	    last_y: 0,
	}
    }

    fn wait_for_enter(&self) {
	let mut stdin = io::stdin();
	let mut stdout = io::stdout();

	write!(stdout, "Press enter...").unwrap();
	stdout.flush().unwrap();

	// Read a single byte and discard
	let _ = stdin.read(&mut [0u8]).unwrap();
    }
    
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
	let pixel = self.im.get_pixel_mut(x, y);
	*pixel = image::Rgb(color.as_u8_array());

	if let Some(a) = self.canvas.as_mut() {
	    a.set_draw_color(color.as_sdl2_color());
	    let _ = a.draw_point(Point::new(x as i32, y as i32));

	    if y != self.last_y {
		a.present();
		self.last_y = y;
	    }
	}
    }

    pub fn save(&self) {
	self.im.save(&CONFIG.output).unwrap();
	println!("Saved image to {}", CONFIG.output);
    }

    pub fn finish_displayed_canvas(&mut self) {
	if let Some(a) = self.canvas.as_mut() {
	    a.present();
	    self.wait_for_enter();
	}
    }
}
