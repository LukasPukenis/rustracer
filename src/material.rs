#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub reflective: f64,
}

impl Material {
    pub fn new() -> Material {
        Material {
            reflective: 0.0,
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
        }
    }
}
