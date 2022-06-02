use crate::camera::Camera;
use crate::gui::BBox;
use crate::gui::PartialRenderMessage;
use crate::gui::Settings;

use crate::material::Material;
use crate::ray::Ray;

use crate::animation;

use crate::loader;
use crate::vec3::Vec3;
use rand::prelude::*;



use std::sync::mpsc;
use std::sync::Arc;

use std::sync::Mutex;

pub trait RenderCallbacks {
    fn progress(&mut self, v: f32);
}

const ambient_r: f64 = 0.1;
const ambient_g: f64 = 0.1;
const ambient_b: f64 = 0.1;

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
    fn set_property(&mut self, prop: animation::AnimationProperty, val: f64);
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

fn renderBlock(
    scene: Arc<Mutex<Scene>>,
    camera: Camera,
    settings: Settings,
    bbox: BBox,
) -> Vec<Pixel> {
    let mut pixels = Vec::new();

    let scnheight = scene.lock().unwrap().height;
    let scnwidth = scene.lock().unwrap().width;
    let _threads = 8; // 2^3 make configurable for the number of threads

    let aspect = 1.0;
    let theta = (camera.fov).to_radians(); // 50mm ff -> 46.8
    let h = (theta / 2.0).tan();
    let viewport_height = 2.0 * h; // todo: parameterize
    let viewport_width = aspect * viewport_height as f64;

    let horizontal = Vec3::new_with(viewport_width as f64, 0.0, 0.0);
    let vertical = Vec3::new_with(0.0, viewport_height as f64, 0.0);
    let origin = camera.pos;
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - camera.dir;

    for j in bbox.x..(bbox.x + bbox.w) {
        for i in bbox.y..(bbox.y + bbox.h) {
            let mut final_color = Vec3::new();
            let mut rng = rand::thread_rng();

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
                x: i as u64,
                y: j as u64,
                color: final_color,
            })
        }
    }

    pixels
}

// fn fillPixelsFromBBox(srcpixels: Vec<Pixel>, dstpixels: Vec<Pixel>, bbox: BBox) {

// }

// todo: static is bad?
pub fn draw(
    scene: Arc<Mutex<Scene>>,
    camera: Camera,
    _thread_cnt: usize,
    _progress_channel: mpsc::Sender<f32>,
    settings: Settings,
    tx: mpsc::Sender<PartialRenderMessage>,
) -> Vec<Pixel> {
    // todo, lets use locking at the top to avoid repetition
    // let scene = scn.lock().unwrap();
    let aspect = 1.0;
    let theta = (camera.fov).to_radians(); // 50mm ff -> 46.8
    let h = (theta / 2.0).tan();
    let viewport_height = 2.0 * h; // todo: parameterize
    let viewport_width = aspect * viewport_height as f64;

    let horizontal = Vec3::new_with(viewport_width as f64, 0.0, 0.0);
    let vertical = Vec3::new_with(0.0, viewport_height as f64, 0.0);
    let origin = camera.pos;
    let _lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - camera.dir;

    let scnheight = scene.lock().unwrap().height;
    let scnwidth = scene.lock().unwrap().width;
    let threads = 8; // 2^3 make configurable for the number of threads

    let final_frame = Vec::new();

    let _progress_full = scnheight * scnwidth;

    let mut bboxes = getBBoxesFor(scnwidth as i32, scnheight as i32, 16);
    bboxes.reverse();

    let pool = threadpool::ThreadPool::new(threads as usize);
    // let mut handles: Vec<std::thread::JoinHandle<_>> = Vec::new();

    let progress = Arc::new(Mutex::new(0));

    let bboxes_len = bboxes.len();

    for bbox in bboxes {
        let scene_clone = scene.clone();
        let tx_clone = tx.clone();
        // let h = std::thread::spawn(move || {
        //     let pixels = renderBlock(scene_clone, camera, settings, bbox);
        //     tx_clone.send(PartialRenderMessage::new(
        //         Arc::new(Mutex::new(pixels)),
        //         bbox,
        //     ))
        // });

        let progress_clone = progress.clone();

        pool.execute(move || {
            let pixels = renderBlock(scene_clone, camera, settings, bbox);
            *progress_clone.lock().unwrap() += 1;

            let p = *progress_clone.lock().unwrap() as f32 / bboxes_len as f32;

            tx_clone
                .send(PartialRenderMessage::new(
                    Arc::new(Mutex::new(pixels)),
                    bbox,
                    p,
                ))
                .unwrap();
        });

        // handles.push(h);
    }

    pool.join();

    // let l = handles.len();
    // for h in handles {
    //     // todo: progress
    //     h.join().unwrap();
    // }

    final_frame
}

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
    if depth <= 0 {
        return Vec3::new_with_all(0.0);
    }

    match collide(r, &scn) {
        Some(collision_data) => {
            let mat = collision_data.1.mat;

            match collision_data.1.kind {
                loader::Kind::Light => Vec3::new_with(mat.color.r, mat.color.g, mat.color.b),
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
                            }
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
                                        if light_intensity < 0.0 {
                                            light_intensity = 0.0;
                                        }
                                    }
                                    loader::Kind::Object => {
                                        // todo: i have no idea why this never happens
                                        // panic!("shoudl happen");
                                    }
                                }
                            }
                        }
                    }

                    let scatter_ray = Ray::new(
                        collision_data.0.point,
                        r.dir.reflect(&collision_data.0.normal),
                    );

                    // for matte
                    // let target =
                    //     collision_data.0.point + collision_data.0.normal + random_point_in_circle();

                    // mirror/metal
                    color * light_intensity + ray_color(&scatter_ray, &scn, depth - 1) * 0.5
                    // todo: hardcoded material number
                }
            }
        }
        None => Vec3::new_with(ambient_r, ambient_g, ambient_b), // todo: hardcoded
    }
}

fn getBBoxesFor(w: i32, h: i32, subdivisions: i32) -> Vec<BBox> {
    let block_w = w / subdivisions;
    let block_h = h / subdivisions;

    let mut bboxes: Vec<BBox> = Vec::new();

    let mut x = 0;
    let mut y = 0;
    let mut limx = subdivisions - 1;
    let mut limy = subdivisions - 1;
    let mut startx = 0;
    let mut starty = 1;

    #[derive(Debug)]
    enum Direction {
        Right,
        Down,
        Left,
        Up,
    }

    let mut dir = Direction::Right;

    for _i in 0..subdivisions * subdivisions {
        bboxes.push(BBox {
            x: x * block_w,
            y: y * block_h,
            w: block_w,
            h: block_h,
        });

        match dir {
            Direction::Right => {
                x += 1;
            }
            Direction::Down => {
                y += 1;
            }
            Direction::Left => {
                x -= 1;
            }
            Direction::Up => {
                y -= 1;
            }
        }

        match dir {
            Direction::Right => {
                if x == limx {
                    dir = Direction::Down;
                    limx -= 1;
                }
            }
            Direction::Down => {
                if y == limy {
                    dir = Direction::Left;
                    limy -= 1;
                }
            }
            Direction::Left => {
                if x == startx {
                    dir = Direction::Up;
                    startx += 1;
                }
            }
            Direction::Up => {
                if y == starty {
                    dir = Direction::Right;
                    starty += 1;
                }
            }
        }
    }

    assert!(bboxes.len() as i32 == subdivisions * subdivisions);

    bboxes
}
#[cfg(test)]
mod tests {
    use super::getBBoxesFor;
    use super::BBox;

    #[test]
    fn test_bbox_generator4() {
        // let b = getBBoxesFor(8, 8, 8);
        // assert_eq!(b.len(), 64);

        let bboxes = getBBoxesFor(4, 4, 4);
        assert_eq!(bboxes.len(), 16);

        let expected = vec![
            BBox::new(0, 0, 1, 1),
            BBox::new(1, 0, 1, 1),
            BBox::new(2, 0, 1, 1),
            BBox::new(3, 0, 1, 1),
            BBox::new(3, 1, 1, 1),
            BBox::new(3, 2, 1, 1),
            BBox::new(3, 3, 1, 1),
            BBox::new(2, 3, 1, 1),
            BBox::new(1, 3, 1, 1),
            BBox::new(0, 3, 1, 1),
            BBox::new(0, 2, 1, 1),
            BBox::new(0, 1, 1, 1),
            BBox::new(1, 1, 1, 1),
            BBox::new(2, 1, 1, 1),
            BBox::new(2, 2, 1, 1),
            BBox::new(1, 2, 1, 1),
        ];

        for (k, _v) in bboxes.iter().enumerate() {
            assert_eq!(bboxes[k], expected[k], "@ index {}", k);
        }
    }
}
