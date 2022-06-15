/**
 * todo: how to make f64 as a generic  properly
 * todo: cant operator ofverload have &mut self?
 */
use std::ops;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn new_with_all(v: f64) -> Vec3 {
        Vec3 { x: v, y: v, z: v }
    }

    pub fn new_with(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn length_squared(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2))
    }

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn neg(&self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /*
        reflect vector around the normal and return a new vector.
        This can be explained as this using a paper and a pencil:
        - draw incoming, normal and reflected vectors
        - reposition the original vector to start at the center,
        this doesn't change a thing however it reveals that the reflected vector
        is original vector plus the vector going upwards to reflected vector. This
        vector is actually made out of two identical vectors and we need to find those.
        - to find that vector we can do a dot product of normal and original vector
        then multiply by the unit normal vector and we will receive a new vector
        that's in the same direction as normal but with magnitude of projection of
        incoming vector on the normal one. Multiply it by two and we have our mystery vector.
        - take the resulting vector which is 2*dot(V,N)*N and add it to the original vector
    */
    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        *self - (*normal * 2.0 * self.dot(normal)) //todo: how to get this formula, I get + instead of -
    }

    #[allow(dead_code)]
    pub fn unit(&mut self) -> Vec3 {
        // todo: self / self.length()
        Vec3 {
            x: self.x / self.length(),
            y: self.y / self.length(),
            z: self.z / self.length(),
        }
    }

    #[allow(dead_code)]
    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn dot(&self, rhs: &Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Div<Vec3> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, k: f64) -> Vec3 {
        Vec3 {
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, k: f64) -> Vec3 {
        Vec3 {
            x: self.x / k,
            y: self.y / k,
            z: self.z / k,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;
    #[test]
    fn test_initialize_zero() {
        assert_eq!(Vec3::new(), Vec3::new_with_all(0.0));
        assert_eq!(Vec3::new(), Vec3::new_with(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Vec3::new_with(1.0, 2.0, 3.0),
            Vec3::new_with(1.0, 1.0, 1.0) + Vec3::new_with(0.0, 1.0, 2.0)
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Vec3::new_with(1.0, 2.0, 3.0),
            Vec3::new_with(5.0, 5.0, 5.0) - Vec3::new_with(4.0, 3.0, 2.0)
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            Vec3::new_with(1.0, 2.0, 3.0),
            Vec3::new_with_all(0.5) * Vec3::new_with(2.0, 4.0, 6.0)
        );
        assert_eq!(Vec3::new_with_all(3.0), Vec3::new_with_all(0.5) * 6.0);
    }
    #[test]

    fn test_div() {
        assert_eq!(
            Vec3::new_with(1.0, 2.0, 3.0),
            Vec3::new_with(10.0, 20.0, 30.0) / Vec3::new_with_all(10.0)
        );
        assert_eq!(Vec3::new_with_all(3.0), Vec3::new_with_all(30.0) / 10.0);
    }

    #[test]
    fn test_length() {
        assert_eq!(0_f64, Vec3::new().length());

        const K: f64 = 0.00001;
        assert!((1.73205 - Vec3::new_with_all(1.0).length()).abs() < K);
        assert!((7.48331 - Vec3::new_with(2.0, 4.0, 6.0).length()).abs() < K);
    }

    #[test]
    fn test_neg() {
        assert_eq!(Vec3::new_with_all(3.0), Vec3::new_with_all(-3.0).neg());
    }

    #[test]
    fn test_cross() {
        assert_eq!(
            Vec3::new_with(-15.0, -2.0, 39.0),
            Vec3::new_with(3.0, -3.0, 1.0).cross(&Vec3::new_with(4.0, 9.0, 2.0))
        );
        assert_eq!(
            Vec3::new(),
            Vec3::new_with(3.0, -3.0, 1.0).cross(&Vec3::new_with(-12.0, 12.0, -4.0))
        );
    }

    #[test]
    fn test_dot() {
        assert_eq!(
            12.0,
            Vec3::new_with(1.0, 2.0, 3.0).dot(&Vec3::new_with(4.0, -5.0, 6.0))
        );
    }
}
