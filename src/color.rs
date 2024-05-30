extern crate sdl2;
use std::ops;

pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub fn blend(&self, other: &Color, factor: f32) -> Color {
	Color {
	    r: self.r*(1.0 - factor) + other.r*factor,
	    g: self.g*(1.0 - factor) + other.g*factor,
	    b: self.b*(1.0 - factor) + other.b*factor,
	}
    }

    pub fn as_u8_array(&self) -> [u8; 3] {
	[self.r as u8, self.g as u8, self.b as u8]
    }

    pub fn as_sdl2_color(&self) -> sdl2::pixels::Color {
	sdl2::pixels::Color::RGB(self.r as u8, self.g as u8, self.b as u8)
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

pub const SNOW_DARK: Color = Color { r: 10.0, g: 60.0, b: 80.0 };
pub const SNOW: Color = Color { r: 255.0, g: 255.0, b: 255.0 };
pub const LAND_DARK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
pub const ROCK: Color = Color { r: 134.0, g: 138.0, b: 103.0 };
pub const FOREST: Color = Color { r: 122.0, g: 132.0, b: 0.0 };
pub const SEA: Color = Color { r: 0.0, g: 42.0, b: 72.0 };
pub const LAND_BLUE: Color = Color { r: 176.0, g: 215.0, b: 253.0 };
pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
pub const WHITE: Color = Color { r: 255.0, g: 255.0, b: 255.0 };
pub const DARK_SKY_BLUE: Color = Color { r: 119.0, g: 181.0, b: 254.0 };
pub const LIGHT_SKY_BLUE: Color = Color { r: 233.0, g: 249.0, b: 255.0 };
