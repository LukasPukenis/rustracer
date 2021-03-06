use glam::Vec3;

use std::ops;

#[derive(Copy, Clone, Debug, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Into<Vec3> for Color {
    fn into(self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Color {
        Color::new(v.x, v.y, v.z)
    }
}

impl ops::Sub<Color> for Color {
    type Output = Color;

    fn sub(self, rhs: Color) -> Color {
        Color {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl ops::Div<Color> for Color {
    type Output = Color;

    fn div(self, rhs: Color) -> Color {
        Color {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b,
        }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Color;

    fn mul(self, k: f32) -> Color {
        Color {
            r: self.r * k,
            g: self.g * k,
            b: self.b * k,
        }
    }
}

impl ops::Div<f32> for Color {
    type Output = Color;

    fn div(self, k: f32) -> Color {
        Color {
            r: self.r / k,
            g: self.g / k,
            b: self.b / k,
        }
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }

    pub fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Material {
    Metal(Metal),
    Lambertian(Lambertian),
    Dielectric(Dielectric),
}

// has a color and fuzz factor which is how  much to scatter the rays
#[derive(Copy, Clone, Debug)]
pub struct Metal {
    pub color: Color,
    pub fuzz: f32,
    pub albedo: f32,
}

// has a color and refraction index by how much to bend the light
#[derive(Copy, Clone, Debug)]
pub struct Dielectric {
    pub color: Color,
    pub refraction: f32,
}

// has a color and albedo which means how much of light it "eats". 0 means - only it's color will be visible
#[derive(Copy, Clone, Debug)]
pub struct Lambertian {
    pub color: Color,
    pub albedo: f32,
}
