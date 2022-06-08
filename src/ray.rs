use crate::vec3::Vec3;

#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, mut dir: Vec3) -> Ray {
        Ray {
            origin: origin,
            dir: dir.unit(),
        }
    }

    pub fn at(self, t: f64) -> Vec3 {
        self.origin + self.dir * t
    }
}
