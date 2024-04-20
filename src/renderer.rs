extern crate image;
use crate::errors::*;
use crate::atlas::Atlas;
use crate::coord::*;
use crate::config::CONFIG;
use std::f32::consts::PI;
use chrono::{DateTime};
use geomorph::*;

const R_EARTH: f32 = 6371000.0;

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
    focus_depth: f32,
}

impl Renderer {
    pub fn sun_position(time: &str, pos: Coord) -> Result<(f32, f32)> {
	let utm = utm::Utm::new(
	    pos.e as f64, pos.n as f64, true, 33, 'W', false);
	let gc : coord::Coord = utm.into();

	let dt = DateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%z")?;
	let ep = dt.timestamp_millis();
	let pos = sun::pos(ep, gc.lat, gc.lon);
	let az  = pos.azimuth;
	let alt = pos.altitude;

	Ok((az as f32, alt as f32))
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
	// Horizontal distance from observer to target at observer height
	let beta: f64 = ((CONFIG.target - CONFIG.observer).abs()/R_EARTH).into();
	let ro: f64 = (observer_height + R_EARTH).into();
	let rt: f64 = (target_height + R_EARTH).into();
	let x = ro*beta.sin();
	let y = (ro*ro - x*x).sqrt();
	let v_middle_angle = ((rt - y)/x).atan() - beta;

	// Vertical angle correction. The direction towards the horizon is lower
	// than the tangent direction from observer. We calculate the difference
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
	    focus_depth: d,
	})
    }

    pub fn land_color(&self, dist: f32, height: f32,
		      dhx: f32, dhy: f32) -> (u8, u8, u8) {

	let land_dark = (0.0, 0.0, 0.0);
	let rock = (134.0, 138.0, 103.0);
	let forest = (122.0, 132.0, 0.0);
	let sea = (0.0, 42.0, 72.0);
	let blue = (176.0, 215.0, 253.0);
        let white = (255.0, 255.0, 255.0);

	// 0 = far, 1 = close
	let blueness = (-CONFIG.rayleigh*0.00003*dist).exp();
	// 0 = hazy, 1 = clear
        let whiteness = (-CONFIG.haziness*0.000002*dist).exp();

	let color;

	let grad = dhx*dhx + dhy*dhy;
	
	if height == 0.0 && grad == 0.0 {
	    // Sea
	    color = sea;
	}
	else {
	    // Land. Determine rock or forest by height above sea and absolute
	    // gradient.
	    let land_color;

	    if (height + grad*100.0) > 800.0 {
		land_color = rock;
	    }
	    else {
		if grad > 0.8 {
		    land_color = rock;
		}
		else {
		    land_color = forest;
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

	    color = (
		land_dark.0*(1.0 - light) + land_color.0*light,
		land_dark.1*(1.0 - light) + land_color.1*light,
		land_dark.2*(1.0 - light) + land_color.2*light,
	    );
	}

	// Add blueness to distant terrain
	let blued = (
            color.0*blueness + blue.0*(1.0 - blueness),
            color.1*blueness + blue.1*(1.0 - blueness),
	    color.2*blueness + blue.2*(1.0 - blueness),
	);

	// Use haziness param to add whiteness to distant terrain
        let whited = (
            (blued.0*whiteness + white.0*(1.0 - whiteness)) as u8,
            (blued.1*whiteness + white.1*(1.0 - whiteness)) as u8,
            (blued.2*whiteness + white.2*(1.0 - whiteness)) as u8,
        );
        return whited;
    }

    pub fn sky_color(&self, angle: f32) -> (u8, u8, u8) {
	let dark_blue = (119.0, 181.0, 254.0);
	let light_blue = (233.0, 249.0, 252.0);
	let white = (255.0, 255.0, 255.0);

	let sky_lum = CONFIG.sky_lum;

	// 0 = light blue, 1 = dark blue
	let lum = (sky_lum*(1.0 - 1.0/(angle + self.vertical_angle_corr).sin())).exp();
	// 0 = white, 1 = blue
	let haze = (0.01*CONFIG.haziness*(1.0 - 1.0/(angle + self.vertical_angle_corr).sin())).exp();

	// Blend light and dark blue, creating gradient of luminance towards
	// the horizon.
	let blended_blue = (light_blue.0*(1.0 - lum) + dark_blue.0*lum,
			    light_blue.1*(1.0 - lum) + dark_blue.1*lum,
			    light_blue.2*(1.0 - lum) + dark_blue.2*lum);

	// Blend whiteness from haze.
	let whited = ((white.0*(1.0 - haze) + blended_blue.0*haze) as u8,
		      (white.1*(1.0 - haze) + blended_blue.1*haze) as u8,
		      (white.2*(1.0 - haze) + blended_blue.2*haze) as u8);

	return whited;
    }

    pub fn render_ray(&mut self, v_angle: f32, ray_end: Coord) -> Option<(Coord, f32)> {
        // Iterate ray
        let mut r = CONFIG.min_depth;

        while r < CONFIG.max_depth {
            // Calculate north and east coordinates
            let c = (ray_end - CONFIG.observer)*(r/CONFIG.max_depth) +
		CONFIG.observer;

            // Calculate height
	    let beta = r/R_EARTH;
	    let alfa = beta + v_angle;
	    let h = (R_EARTH + self.observer_height)*
		(beta.cos() + beta.sin()*alfa.tan()) - R_EARTH;

            if h > 2600.0 {
                // Above highest terrain level on Norwegian mainland
                return None;
	    }

            let mut ret : Result<f32> = Err(Error::Generic().into());
	    if r < self.r10 {
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
		    if r < self.r10 {
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
	    if r < self.dr_min_range {
		r += self.dr_min;
	    }
	    else if r > self.dr_max_range {
		r += self.dr_max;
	    }
	    else {
		r += r/self.dr_factor;
	    }
	}

	None
    }
    
    pub fn render(&mut self) -> Result<()> {
	let mut im = image::ImageBuffer::new(CONFIG.width, CONFIG.height);

        let o = CONFIG.observer;

        for y in 0..CONFIG.height {
            // Calculate vertical angle
            let v_angle: f32 = self.vertical_middle_angle +
		(((CONFIG.height as f32)/2.0 - (y as f32))/self.focus_depth).atan();
//	    println!("{}", v_angle);

            println!("Line {}", y);

//            for x in 0..CONFIG.width {
            for x in 0..CONFIG.width {
		// Calculate directional angle
                let h_angle = self.horizontal_middle_angle +
		    (((CONFIG.width as f32)/2.0 - (x as f32))/self.focus_depth).atan();
                // Calculate ray endpoint
                let ray_end = Coord::from_polar(CONFIG.max_depth, h_angle) + o;

		let color;

		let ray = self.render_ray(v_angle, ray_end);

		if let Some((coord, r)) = ray {
		    // Found land
                    // Calculate straight distance (can be ommitted)
		    //   r_straight = R_EARTH*(r/R_EARTH).sin()/(r/r_earth + v_angle).cos();
                    // Calculate land color
		    let mut ret : Result<(f32, f32, f32)> = Err(Error::Generic().into());
		    if r < self.r10 {
			// Try 1m resolution lookup if we are close
			ret = self.atlas1.lookup_with_gradient(&coord);
		    }
		    
		    if let Err(_) = ret {
			// Fallback to 10m resolution
			ret = self.atlas10.lookup_with_gradient(&coord);
		    }
	    
		    if let Ok((h, dx, dy)) = ret {
			color = self.land_color(r, h, dx, dy);
		    }
		    else {
			// Fallback to sea
			color = self.land_color(r, 0.0, 0.0, 0.0);
		    }
		}
		else {
		    // Land was not found, assume sky
		    color = self.sky_color(v_angle);

		}

		let pixel = im.get_pixel_mut(x, y);
		*pixel = image::Rgb([color.0, color.1, color.2]);
            }
	}

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

	    let ray = self.render_ray(v_angle, ray_end);

	    if let Some((coord, _)) = ray {
		return Ok(coord);
	    }
        }
	Err(Error::HorizonNotFound().into())
    }
}
