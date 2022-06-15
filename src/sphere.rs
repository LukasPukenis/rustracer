use crate::animation::AnimationProperty;
use crate::ray::Ray;
use crate::scene::random_point_in_circle;
use crate::scene::CollisionData;
use crate::scene::Face;
use crate::scene::Hitable;

use crate::vec3::Vec3;

#[derive(Copy, Clone)]
pub struct Sphere {
    pub pos: Vec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(pos: Vec3, radius: f64) -> Sphere {
        Sphere { pos, radius }
    }
}

const THRESHOLD: f64 = 0.001; //0.001;

impl Hitable for Sphere {
    fn get_random_point(&self) -> Vec3 {
        random_point_in_circle() * self.radius
    }

    fn set_property(&mut self, prop: AnimationProperty, val: f64) {
        match prop {
            AnimationProperty::X => self.pos.x = val,
            AnimationProperty::Y => self.pos.y = val,
            AnimationProperty::Z => self.pos.z = val,
            AnimationProperty::RADIUS => self.radius = val,
        }
    }
    fn pos(&self) -> Vec3 {
        self.pos
    }

    fn pos_mut(&mut self) -> &mut Vec3 {
        &mut self.pos
    }

    fn hit(&self, r: &Ray) -> Option<CollisionData> {
        let oc = r.origin - self.pos;
        let a = r.dir.dot(&r.dir);
        let b = 2.0 * oc.dot(&r.dir);
        let c = oc.dot(&oc) - self.radius.powi(2);
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            None
        } else {
            let solution = (-b - discriminant.sqrt()) / (2.0 * a);

            if solution <= THRESHOLD {
                return None;
            }

            let point = r.at(solution);
            let mut normal = (point - self.pos) / self.radius; // todo: same as .unit() ?
            let face: Face;

            if normal.dot(&r.dir) > 0.0 {
                face = Face::Back;
                normal = normal.neg();
            } else {
                face = Face::Front;
            }

            Some(CollisionData {
                face,
                normal,
                point,
            })
        }
    }
}
