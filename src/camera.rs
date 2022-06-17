use glam::Vec3;

#[derive(Copy, Clone)]
pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub fov: f32,
}

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, fov: f32) -> Camera {
        Camera { pos, dir, fov }
    }
}
