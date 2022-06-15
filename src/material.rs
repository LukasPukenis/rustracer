#[derive(Copy, Clone, Debug, Default)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
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
    pub fuzz: f64,
    pub albedo: f64,
}

// has a color and refraction index by how much to bend the light
#[derive(Copy, Clone, Debug)]
pub struct Dielectric {
    pub color: Color,
    pub refraction: f64,
}

// has a color and albedo which means how much of light it "eats". 0 means - only it's color will be visible
#[derive(Copy, Clone, Debug)]
pub struct Lambertian {
    pub color: Color,
    pub albedo: f64,
}
