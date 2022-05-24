use crate::camera::Camera;
use crate::material::Material;
use crate::ray::Ray;
use crate::gui::Settings;

use crate::vec3::Vec3;
use rand::prelude::*;

use crate::loader;

use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;

pub trait RenderCallbacks {
    fn progress(&mut self, v: f32);
}

pub struct Pixel {
    pub x: u64,
    pub y: u64,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct Object {
    mat: Material,
    pub geometry: Arc<Mutex<dyn Hitable>>, // todo: why having Rc here and cloning doesn't work in a spawned thread?
    kind: loader::Kind,
}

pub struct Scene {
    width: u64,
    height: u64,
    objects: Vec<Object>,
    lights: Vec<Object>,
}

impl Scene {
    pub fn new(width: u64, height: u64) -> Scene {
        Scene {
            width,
            height,
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    // todo: how to get rid of mutex in args?
    pub fn add_object(&mut self, g: Arc<Mutex<dyn Hitable>>, m: Material) {
        self.objects.push(Object {
            kind: loader::Kind::Object,
            mat: m,
            geometry: g,
        });
    }

    pub fn add_light(&mut self, g: Arc<Mutex<dyn Hitable>>, m: Material) {
        self.lights.push(Object {
            kind: loader::Kind::Light,
            mat: m,
            geometry: g,
        });
    }

    // todo: as ref? &[Object]
    pub fn lights(&self) -> &Vec<Object> {
        &self.lights
    }

    pub fn objects(&self) -> &Vec<Object> {
        &self.objects
    }
}

pub enum Face {
    Front,
    Back,
}

pub struct CollisionData {
    pub face: Face,
    pub normal: Vec3,
    pub point: Vec3,
}

pub trait Hitable: Send + Sync {
    // todo: send sync required?
    fn hit(&self, r: &Ray) -> Option<CollisionData>;
    fn pos(&self) -> Vec3;
    fn pos_mut(&mut self) -> &mut Vec3;
}

fn random_point_in_circle() -> Vec3 {
    let mut rng = rand::thread_rng();

    loop {
        let x: f64 = 1.0 - rng.gen::<f64>() * 2.0;
        let y: f64 = 1.0 - rng.gen::<f64>() * 2.0;
        let z: f64 = 1.0 - rng.gen::<f64>() * 2.0;
        let v = Vec3::new_with(x, y, z);

        // todo: might be wrong
        if v.length() * v.length() >= 1.0 {
            continue;
        }

        return v;
    }
}

pub fn draw(scene: Arc<Mutex<Scene>>, camera: &Camera, _thread_cnt: usize, progress_channel: mpsc::Sender<f32>, settings: Settings) -> Vec<Pixel> {
    // todo, lets use locking at the top to avoid repetition
    // let scene = scn.lock().unwrap();
    let aspect = 1.0;
    let theta = (camera.fov).to_radians(); // 50mm ff -> 46.8
    let h = (theta/2.0).tan();
    let viewport_height = 2.0 * h; // todo: parameterize
    let viewport_width = aspect * viewport_height as f64;

    let horizontal = Vec3::new_with(viewport_width as f64, 0.0, 0.0);
    let vertical = Vec3::new_with(0.0, viewport_height as f64, 0.0);
    let origin = camera.pos;
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - camera.dir;

    fn collide<'a>(r: &Ray, scn: &Scene) -> Option<(CollisionData, Object)> {
        let mut closest_obj: Option<Object> = None;
        let mut closest_data: Option<CollisionData> = None;
        let mut closest_distance: f64 = 99999999999.9;

        for obj in scn.objects.iter() {
            match obj.geometry.lock().unwrap().hit(r) {
                None => continue,
                Some(data) => {
                    let distance = (r.origin - data.point).length();
                    if distance < closest_distance {
                        closest_obj = Some(obj.clone());
                        closest_data = Some(data);
                        closest_distance = distance;
                    }
                }
            }
        }

        for light in scn.lights.iter() {
            match light.geometry.lock().unwrap().hit(r) {
                None => continue,
                Some(data) => {
                    let distance = (r.origin - data.point).length();
                    if distance < closest_distance {
                        // println!("distance2 check from {} to {}", closest_distance, distance);
                        closest_obj = Some(light.clone());
                        closest_data = Some(data);
                        closest_distance = distance;
                    }
                }
            }
        }

        match closest_obj {
            Some(obj) => {
                // println!("{}", closest_distance);
                Some((closest_data.unwrap(), obj))
            }
            None => None,
        }
    }

    /**
     * We hit the scene with a ray, if it hit something then we take the objects material into
     * account how to render it but also do a shadow ray towards all sources of light to see if we should
     * light the pixel. We also launch scatter rays(todo: should be abstracted, as different materials have it varying,
     * like metal reflects almost perfectly instead of randomly)
     */

     // todo: Vec3->Color
    fn ray_color(r: &Ray, scn: &Scene, depth: i16) -> Vec3 {
        // todo: this is needed?
        // if depth <= 0 {
        //     return Vec3::new_with_all(0.0);
        // }

        match collide(r, &scn) {
            Some(collision_data) => {
                let mat = collision_data.1.mat;

                match collision_data.1.kind {
                    loader::Kind::Light => {
                        Vec3::new_with(mat.color.r, mat.color.g, mat.color.b)
                    },
                    loader::Kind::Object => {
                        let collision_point = collision_data.0.point;
                        let collision_normal = collision_data.0.normal;

                        let color = Vec3::new_with(mat.color.r, mat.color.g, mat.color.b);
                        let mut light_intensity = 0.0;

                        // nowe as we've hit the object in the scene, we need to determine
                        // it's relation to the light sources, it might be in the shadow or might be
                        // lit. In order to find that out we collide another ray from collision point towards
                        // all the light sources in the scene and light the pixel accordingly
                        // we do not care about light's color at the moment

                        for light in scn.lights() {
                            let shadow_dir = light.geometry.lock().unwrap().pos() - collision_point;
                            let shadow_ray = Ray::new(collision_point, shadow_dir);

                            match collide(&shadow_ray, &scn) {
                                None => {
                                    panic!("should happen")
                                },
                                Some(shadow_coll) => {
                                    match shadow_coll.1.kind {
                                        loader::Kind::Light => {
                                            let n = shadow_coll.0.point.clone().unit();
                                            let m = collision_normal.clone().unit();
                                            let dot = m.dot(&n);
                                            light_intensity += dot;
                                            if light_intensity > 1.0 {
                                                light_intensity = 1.0;
                                            }
                                        },
                                        loader::Kind::Object => {
                                            // todo: i have no idea why this never happens
                                            // panic!("shoudl happen");
                                        },
                                    }
                                }
                            }
                        }

                        color * light_intensity
                    },
                }
            },
            None => Vec3::new_with(0.3, 0.3, 0.3) // todo: hardcoded
        }
    }

    let scnheight = scene.lock().unwrap().height;
    let scnwidth = scene.lock().unwrap().width;

    let mut pixels = Vec::new();

    let progress_full = scnheight * scnwidth;

    for j in 0..scnheight - 1 {
        for i in 0..scnwidth - 1 {
            let mut final_color = Vec3::new();

            let mut rng = rand::thread_rng();

            let progress = (j as f32 * scnwidth as f32 + i as f32) / progress_full as f32;
            progress_channel.send(progress).unwrap();

            // todo: should be easy to parallelize by taking the screen in blocks for each thread
            for _ in 0..settings.samples {
                let xoff: f64 = 1.0 - (2.0 * rng.gen::<f64>());
                let yoff: f64 = 1.0 - (2.0 * rng.gen::<f64>());

                let u = (i as f64 + xoff) / ((scnwidth - 1) as f64);
                let v = (j as f64 + yoff) / ((scnheight - 1) as f64);
                let r = Ray::new(
                    origin,
                    lower_left_corner + horizontal * u as f64 + vertical * v as f64 - origin,
                );

                // todo: spaghetti
                let color = ray_color(&r, &scene.clone().lock().unwrap(), 100);
                final_color = final_color + color;
            }

            final_color = final_color / settings.samples as f64;
            final_color.x = final_color.x.clamp(0.0, 1.0);
            final_color.y = final_color.y.clamp(0.0, 1.0);
            final_color.z = final_color.z.clamp(0.0, 1.0);

            pixels.push(Pixel {
                x: i,
                y: j,
                color: final_color,
            })
        }
    }

    pixels
}
