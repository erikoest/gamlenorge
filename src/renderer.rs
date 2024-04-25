extern crate image;
use std::ops;
use crate::errors::*;
use crate::atlas::Atlas;
use crate::coord::*;
use crate::config::CONFIG;
use crate::progress::PROGRESS;
use std::f32::consts::PI;
use chrono::{DateTime};
use geomorph::*;
use rand::Rng;

const R_EARTH: f32 = 6371000.0;

struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    fn blend(&self, other: &Color, factor: f32) -> Color {
	Color {
	    r: self.r*(1.0 - factor) + other.r*factor,
	    g: self.g*(1.0 - factor) + other.g*factor,
	    b: self.b*(1.0 - factor) + other.b*factor,
	}
    }

    fn as_u8_array(&self) -> [u8; 3] {
	[self.r as u8, self.g as u8, self.b as u8]
    }
}

impl ops::AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Self) {
	self.r += rhs.r;
	self.g += rhs.g;
	self.b += rhs.b;
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, _rhs: Color) -> Color {
	Color { r: self.r + _rhs.r, g: self.g + _rhs.g, b: self.b + _rhs.b }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, _rhs: f32) -> Color {
	Color { r: self.r*_rhs, g: self.g*_rhs, b: self.b*_rhs }
    }
}

const SNOW_DARK: Color = Color { r: 10.0, g: 60.0, b: 80.0 };
const SNOW: Color = Color { r: 255.0, g: 255.0, b: 255.0 };
const LAND_DARK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
const ROCK: Color = Color { r: 134.0, g: 138.0, b: 103.0 };
const FOREST: Color = Color { r: 122.0, g: 132.0, b: 0.0 };
const SEA: Color = Color { r: 0.0, g: 42.0, b: 72.0 };
const LAND_BLUE: Color = Color { r: 176.0, g: 215.0, b: 253.0 };
const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
const WHITE: Color = Color { r: 255.0, g: 255.0, b: 255.0 };
const DARK_SKY_BLUE: Color = Color { r: 119.0, g: 181.0, b: 254.0 };
const LIGHT_SKY_BLUE: Color = Color { r: 233.0, g: 249.0, b: 255.0 };

pub struct Renderer {
    atlas1: Atlas,
    atlas10: Atlas,
    sun_ray: Coord3,
    observer_height: f32,
    horizontal_middle_angle: f32,
    vertical_middle_angle: f32,
    vertical_angle_corr: f32,
    r10: f32,
    dr_min: f32,
    dr_max: f32,
    dr_factor: f32,
    dr_min_range: f32,
    dr_max_range: f32,
    sea_min_reflection_angle: f32,
    focus_depth: f32,
}

impl Renderer {
    pub fn sun_position(time: &str, pos: Coord) -> Result<(f32, f32)> {
	let utm = utm::Utm::new(
	    pos.e as f64, pos.n as f64, true, 33, 'W', false);
	let gc : coord::Coord = utm.into();

	let res = DateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%z");
	if let Ok(dt) = res {
	    let ep = dt.timestamp_millis();
	    let pos = sun::pos(ep, gc.lat, gc.lon);
	    let az  = pos.azimuth;
	    let alt = pos.altitude;

	    Ok((az as f32, alt as f32))
	}
	else {
	    return Err(Error::InvalidTimestamp(time.to_string()).into());
	}
    }

    pub fn new(atlas1: Atlas, atlas10: Atlas) -> Result<Self> {
	// Pre-calculate as much as we can before start.

	// Calculate sun ray directional unit vector based on horizontal and
	// vertical angle
	let (az, alt) = Renderer::sun_position(&CONFIG.time, CONFIG.observer)?;
	let sun_ray = Coord3::new(0.0, 1.0, 0.0).rot_e(alt).rot_h(-az);

        // Observer ground height
	let observer_height = atlas10.lookup(&CONFIG.observer)? +
	    CONFIG.observer_height_offset;

        // Target ground height
	let target_height = atlas10.lookup(&CONFIG.target)? +
	    CONFIG.target_height_offset;

        // Middle directional angle
	let diff = CONFIG.target - CONFIG.observer;
	let mut h_middle_angle;
        if diff.e.abs() > diff.n.abs() {
            h_middle_angle = (diff.n/diff.e).atan();
            if diff.e < 0.0 {
                if h_middle_angle <= 0.0 {
                    h_middle_angle += PI;
		}
                else {
                    h_middle_angle = h_middle_angle - PI;
		}
	    }
	}
        else {
            h_middle_angle = 0.5*PI - (diff.e/diff.n).atan();
	    if diff.n < 0.0 {
		h_middle_angle = h_middle_angle - PI;
	    }
	}

        // Middle vertical angle. The formula includes ground curvature
	// Horizontal distance from observer to target at observer height.
	let beta: f64 = ((CONFIG.target - CONFIG.observer).abs()/R_EARTH).into();
	let ro: f64 = (observer_height + R_EARTH).into();
	let rt: f64 = (target_height + R_EARTH).into();
	let x = ro*beta.sin();
	let y = (ro*ro - x*x).sqrt();
	let v_middle_angle = ((rt - y)/x).atan() - beta;

	// Vertical angle correction. The direction towards the horizon is
	// lower than the tangent direction from observer. We calculate the
	// difference.
	let v_angle_corr = (R_EARTH/(R_EARTH + observer_height)).acos();

	let dr_min = 0.9;
	let dr_max = 30.0;
	let dr_factor = (CONFIG.width as f32)/(3.0*CONFIG.width_angle.tan());
	let dr_min_range = dr_min*dr_factor;
	let dr_max_range = dr_max*dr_factor;

        // Depth of viewer from image
        let d = (CONFIG.width as f32)/(2.0*CONFIG.width_angle.tan());

	// Distance when we switch from 1m to 10m samples. We do this when
	// the angle of one pixle is approximately 8m
	let r10 = 8.0*d;

	Ok(Self {
	    atlas1: atlas1,
	    atlas10: atlas10,
	    sun_ray: sun_ray,
	    observer_height: observer_height,
	    horizontal_middle_angle: h_middle_angle,
	    vertical_middle_angle: (v_middle_angle as f32),
	    vertical_angle_corr: (v_angle_corr as f32),
	    r10: r10,
	    dr_min: dr_min,
	    dr_max: dr_max,
	    dr_factor: dr_factor,
	    dr_min_range: dr_min_range,
	    dr_max_range: dr_max_range,
	    sea_min_reflection_angle: 0.5_f32.to_radians(),
	    focus_depth: d,
	})
    }

    fn land_color(&mut self,
		  dist: f32,
		  total_dist: f32,
		  angle: f32,
		  coord: Coord) -> Color {

	// Calculate land color
	let height;
	let dhx;
	let dhy;

	let mut ret : Result<(f32, f32, f32)> = Err(Error::Generic().into());

	if total_dist < self.r10 {
	    // Try 1m resolution lookup if we are close
	    ret = self.atlas1.lookup_with_gradient(&coord);
	}

	if let Err(_) = ret {
	    // Fallback to 10m resolution
	    ret = self.atlas10.lookup_with_gradient(&coord);
	}

	if let Ok((h, dx, dy)) = ret {
	    dhx = dx;
	    dhy = dy;
	    height = h;
	}
	else {
	    // Fallback to sea
	    dhx = 0.0;
	    dhy = 0.0;
	    height = 0.0;
	}

	// 0 = blue terrain far away
	// 1 = sharp colors at close distance
	let blueness = (-CONFIG.rayleigh*0.00003*dist).exp();
	// 0 = hazy, 1 = clear
        let whiteness = (-CONFIG.haziness*0.000002*dist).exp();

	let color;

	let grad = dhx*dhx + dhy*dhy;
	
	if height <= CONFIG.water_level {
	    // Water surface. Continue tracing the reflected ray, using
	    // the inverse angle corrected by curvature due to distance.
	    let mut r_angle = dist/R_EARTH - angle;

	    if r_angle < self.sea_min_reflection_angle {
		r_angle = self.sea_min_reflection_angle;
	    }

	    // Blend reflection and flat sea color
	    let n = CONFIG.water_reflection_iterations;
	    let seamix;

	    if n > 0 && CONFIG.water_shininess != 0.0 {
		let mut rcolor = BLACK;
		let re1 = coord - CONFIG.observer;
		let re2 = re1*(CONFIG.max_depth/re1.abs()) + coord;
		let mut rng = rand::thread_rng();
		let afuzz = 0.01*CONFIG.water_ripples;
		let range = afuzz*0.5*PI;

		for _ in 0..n {
		    let rafuzz = rng.gen::<f32>()*range + r_angle*(1.0 - afuzz);
		    let ray = self.render_ray(rafuzz, total_dist, coord,
					      CONFIG.water_level + 1.0, re2);
		    rcolor += self.find_color(ray, total_dist, rafuzz);
		}
		rcolor = rcolor*(1.0/(n as f32));
		seamix = SEA.blend(&rcolor, CONFIG.water_shininess);
	    }
	    else {
		seamix = SEA;
	    }

	    // Then, use Schlick's approximation to calculate reflection rate
	    // of water.
	    let r0 = 0.0200593121995248; // ((1.33 - 1)/(1.33 + 1))^2
	    let r = r0 + (1.0 - r0)*(1.0 - (0.5*PI - r_angle).cos()).powi(5);

	    color = seamix*r;
	}
	else {
	    // Land. Determine rock or forest by height above sea and absolute
	    // gradient.
	    let land_color;
	    let dark_color;

	    if (height - grad*200.0) > CONFIG.snow_limit {
		land_color = SNOW;
		dark_color = SNOW_DARK;
	    }
	    else {
		dark_color = LAND_DARK;
		if (height + grad*100.0) > CONFIG.green_limit {
		    land_color = ROCK;
		}
		else {
		    if grad > 0.8 {
			land_color = ROCK;
		    }
		    else {
			land_color = FOREST;
		    }
		}
	    }

	    /*
	    Calculate shade of terrain as the cosine of angle between terrain
	    normal and sunlight ray.
	    g:       gradient vector (normal to ground plane) [-dy, -dx, dx*dy]
	    sun_ray: sun ray vector (unit length)
            v:       angle between g and sun_ray
            shade:   cos(v) = g.s/(|g|*|s|)  [0 = shade, 1 = light]
	     */
	    let g = Coord3::new(-dhx, -dhy, 1.0);
	    let light = ((g.dot(self.sun_ray))/g.abs()).max(0.0);

	    color = dark_color.blend(&land_color, light);
	}

	// Add blueness to distant terrain
	let blued = LAND_BLUE.blend(&color, blueness);

	// Use haziness param to add whiteness to distant terrain
        let whited = WHITE.blend(&blued, whiteness);

        return whited;
    }

    fn sky_color(&self, angle: f32) -> Color {

	let sky_lum = 0.1*CONFIG.sky_lum;

	// 0 = light blue, 1 = dark blue
	let lum = (sky_lum*(1.0 - 1.0/(angle + self.vertical_angle_corr).sin())).exp();
	// 0 = white, 1 = blue
	let haze = (0.01*CONFIG.haziness*(1.0 - 1.0/(angle + self.vertical_angle_corr).sin())).exp();

	// Blend light and dark blue, creating gradient of luminance towards
	// the horizon.
	let blended_blue = LIGHT_SKY_BLUE.blend(&DARK_SKY_BLUE, lum);

	// Blend whiteness from haze.
	let whited = WHITE.blend(&blended_blue, haze);

	return whited;
    }

    fn find_color(&mut self,
		  ray_output: Option<(Coord, f32)>,
		  passed_dist: f32,
		  v_angle: f32) -> Color {
	if let Some((coord, r)) = ray_output {
	    // Found land
            // Calculate straight distance (can be ommitted)
	    //   r_straight = R_EARTH*(r/R_EARTH).sin()/(r/r_earth + v_angle).cos();
	    return self.land_color(r, passed_dist + r, v_angle, coord);
	}
	else {
	    // Land was not found, assume sky
	    return self.sky_color(v_angle);
	}
    }

    pub fn render_ray(&mut self,
		      v_angle: f32,
		      passed_dist: f32,
		      observer: Coord,
		      observer_height: f32,
		      ray_end: Coord) -> Option<(Coord, f32)> {
        // Iterate ray
        let mut r = CONFIG.min_depth;

        while r < CONFIG.max_depth {
            // Calculate north and east coordinates
            let c = (ray_end - observer)*(r/CONFIG.max_depth) + observer;
	    let total_dist = passed_dist + r;

            // Calculate height
	    let beta = r/R_EARTH;
	    let alfa = beta + v_angle;
	    let h = (R_EARTH + observer_height)*
		(beta.cos() + beta.sin()*alfa.tan()) - R_EARTH;

            if h > 2600.0 {
                // Above highest terrain level on Norwegian mainland
                return None;
	    }

            let mut ret : Result<f32> = Err(Error::Generic().into());
	    if total_dist < self.r10 {
		// Try 1m resolution lookup if we are close
		ret = self.atlas1.lookup(&c);
	    }

	    if let Err(_) = ret {
		// Fallback to 10m resolution
		ret = self.atlas10.lookup(&c);
	    }
	    
	    if let Ok(land_height) = ret {
                if h < land_height {
                    // Found land. If we are close, check if the 1m maps are
		    // loaded. If not, load them, go back 50m and continue
		    // tracing
		    if total_dist < self.r10 {
			if self.atlas1.has_maps(&c) &&
			    !self.atlas1.has_images(&c) {
			    let _ = self.atlas1.load_images(&c);
			    r -= 50.0;
			    continue;
			}
		    }

		    return Some((c, r));
		}
	    }
	    else {
		// Left map. Assume sea level.
		if h < 0.0 {
		    return Some((c, r));
		}
	    }

	    // Sky, step up ray distance, then continue
	    if total_dist < self.dr_min_range {
		r += self.dr_min;
	    }
	    else if total_dist > self.dr_max_range {
		r += self.dr_max;
	    }
	    else {
		r += total_dist/self.dr_factor;
	    }
	}

	None
    }

    pub fn render(&mut self) -> Result<()> {
	let mut im = image::ImageBuffer::new(CONFIG.width, CONFIG.height);

        let o = CONFIG.observer;

	PROGRESS.set_length(CONFIG.height.into());

        for y in 0..CONFIG.height {
            // Calculate vertical angle
            let v_angle: f32 = self.vertical_middle_angle +
		(((CONFIG.height as f32)/2.0 - (y as f32))/self.focus_depth).atan();
//            println!("Line {}", y);

            for x in 0..CONFIG.width {
		// Calculate directional angle
                let h_angle = self.horizontal_middle_angle +
		    (((CONFIG.width as f32)/2.0 - (x as f32))/self.focus_depth).atan();
                // Calculate ray endpoint
                let ray_end = Coord::from_polar(CONFIG.max_depth, h_angle) + o;

		let ray = self.render_ray(v_angle, 0.0, CONFIG.observer, self.observer_height, ray_end);
		let color = self.find_color(ray, 0.0, v_angle);

		let pixel = im.get_pixel_mut(x, y);
		*pixel = image::Rgb(color.as_u8_array());
            }
	    PROGRESS.inc(1);
	}

	PROGRESS.finish();

	im.save(&CONFIG.output).unwrap();
	println!("Saved image to {}", CONFIG.output);

	Ok(())
    }

    pub fn find_horizon(&mut self) -> Result<Coord> {
        let o = CONFIG.observer;

        for y in 0..CONFIG.height {
            // Calculate vertical angle
            let v_angle: f32 = self.vertical_middle_angle +
		(((CONFIG.height as f32)/2.0 - (y as f32))/self.focus_depth).atan();
            // Calculate ray endpoint
            let ray_end = Coord::from_polar(CONFIG.max_depth,
					    self.horizontal_middle_angle) + o;

	    let ray = self.render_ray(v_angle, 0.0, CONFIG.observer, self.observer_height, ray_end);

	    if let Some((coord, _)) = ray {
		return Ok(coord);
	    }
        }
	Err(Error::HorizonNotFound().into())
    }
}
