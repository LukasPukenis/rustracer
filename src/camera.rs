use glam::Vec3;

#[derive(Copy, Clone)]
pub struct Camera {
    pub pos: Vec3,
    pub lookat: Vec3,
    pub fov: f32,
}

impl Camera {
    pub fn new(pos: Vec3, lookat: Vec3, fov: f32) -> Camera {
        Camera { pos, lookat, fov }
    }
}
