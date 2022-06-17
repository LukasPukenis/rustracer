use crate::app::PartialRenderMessagePixels;
use crate::camera::Camera;

use crate::material;
use crate::material::Material;
use crate::ray::Ray;

use crate::app;
use crate::loader;
use crate::material::Color;
use glam::Vec3;
use rand::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::app::BBox;
use crate::app::PartialRenderMessage;

pub trait RenderCallbacks {
    fn progress(&mut self, v: f32);
}

const AMBIENT_R: f32 = 0.0;
const AMBIENT_G: f32 = 0.0;
const AMBIENT_B: f32 = 0.0;

pub struct Pixel {
    pub x: u64,
    pub y: u64,
    pub color: material::Color,
}

#[derive(Clone)]
pub struct Object {
    mat: Material,
    pub geometry: Arc<dyn Hitable>,
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

    pub fn add_object(&mut self, g: Arc<dyn Hitable>, m: Material) {
        self.objects.push(Object {
            kind: loader::Kind::Object,
            mat: m,
            geometry: g,
        });
    }

    pub fn add_light(&mut self, g: Arc<dyn Hitable>, m: Material) {
        self.lights.push(Object {
            kind: loader::Kind::Light,
            mat: m,
            geometry: g,
        });
    }

    // todo: slice?
    pub fn lights(&self) -> &Vec<Object> {
        &self.lights
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
    // todo: should be split into hitable and other traits
    fn hit(&self, r: &Ray) -> Option<CollisionData>;
    fn pos(&self) -> Vec3;
    fn pos_mut(&mut self) -> &mut Vec3;
    fn get_random_point(&self) -> Vec3;
}

pub fn random_point_in_circle() -> Vec3 {
    let mut rng = rand::thread_rng();

    loop {
        let x: f32 = 1.0 - rng.gen::<f32>() * 2.0;
        let y: f32 = 1.0 - rng.gen::<f32>() * 2.0;
        let z: f32 = 1.0 - rng.gen::<f32>() * 2.0;
        let v = Vec3::new(x, y, z);

        if v.length_squared() >= 1.0 {
            continue;
        }

        return v;
    }
}

fn render_block(
    scene: Arc<Scene>,
    camera: Camera,
    settings: app::Settings,
    bbox: BBox,
    tx: mpsc::Sender<f64>,
) -> Vec<Pixel> {
    let mut pixels = Vec::new();

    let scnheight = scene.height;
    let scnwidth = scene.width;

    let aspect = 1.0;
    let theta = (camera.fov).to_radians(); // 50mm ff -> 46.8
    let h = (theta / 2.0).tan();
    let viewport_height = 2.0 * h;
    let viewport_width = aspect * viewport_height as f64;

    let horizontal = Vec3::new(viewport_width as f32, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height as f32, 0.0);
    let origin = camera.pos;
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - camera.dir;

    for j in bbox.x..(bbox.x + bbox.w) {
        for i in bbox.y..(bbox.y + bbox.h) {
            let mut final_color = Color::default();
            let mut rng = rand::thread_rng();

            for _ in 0..settings.samples {
                let xoff: f32 = 1.0 - (2.0 * rng.gen::<f32>());
                let yoff: f32 = 1.0 - (2.0 * rng.gen::<f32>());

                let u = (i as f32 + xoff) / ((scnwidth - 1) as f32);
                let v = (j as f32 + yoff) / ((scnheight - 1) as f32);
                let r = Ray::new(
                    origin,
                    lower_left_corner + horizontal * u as f32 + vertical * v as f32 - origin,
                );

                let color = ray_color(&r, &scene.clone(), 100, settings.shadow_samples);
                final_color = final_color + color;
            }

            final_color = final_color / settings.samples as f32;
            final_color.r = final_color.r.clamp(0.0, 1.0);
            final_color.g = final_color.g.clamp(0.0, 1.0);
            final_color.b = final_color.b.clamp(0.0, 1.0);

            pixels.push(Pixel {
                x: i as u64,
                y: j as u64,
                color: final_color,
            })
        }
    }

    // todo: could be simplified if not sending inside the loop, another place listens for 1.0
    tx.send(1.0).unwrap();

    pixels
}

pub fn draw(
    scene: Arc<Scene>,
    camera: Camera,
    settings: app::Settings,
    tx: mpsc::Sender<PartialRenderMessage>,
) {
    let aspect = 1.0;
    let theta = (camera.fov).to_radians(); // 50mm ff -> 46.8 // todo: show focal length and angle in gui
    let h = (theta / 2.0).tan();
    let viewport_height = 2.0 * h;
    let viewport_width = aspect * viewport_height as f32;

    let horizontal = Vec3::new(viewport_width as f32, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height as f32, 0.0);
    let origin = camera.pos;
    let _lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - camera.dir;

    let scnheight = scene.height;
    let scnwidth = scene.width;

    let _progress_full = scnheight * scnwidth;

    // depending on the renderer size, increasing this produces multithreaded operation
    let mut bboxes = get_bboxes_for(scnwidth as i32, scnheight as i32, settings.bboxes as i32);
    bboxes.reverse();

    let pool = threadpool::ThreadPool::new(settings.threads as usize);
    // let mut handles: Vec<std::thread::JoinHandle<_>> = Vec::new();

    let progress = Arc::new(Mutex::new(0));

    let bboxes_len = bboxes.len();
    let progress_ratio = 1.0 / bboxes_len as f64;
    println!("{} bboxes for {} threads", bboxes_len, settings.threads);

    let (progtx, progrx) = mpsc::channel();

    let total_progress = Arc::new(Mutex::new(0.0));

    let tx_clone3 = tx.clone();
    std::thread::spawn(move || loop {
        match progrx.try_recv() {
            Ok(p) => {
                let progress: f64 = *total_progress.lock().unwrap() + p * progress_ratio;
                let progress_bar_symbols = 30;
                let ratio: f64 = 1.0 / progress_bar_symbols as f64;

                let symbol_count: usize = (progress / ratio) as usize;
                let progress_str: String = (0..symbol_count).into_iter().map(|_| ".").collect();

                let padding: String = (0..(progress_bar_symbols - symbol_count))
                    .into_iter()
                    .map(|_| " ")
                    .collect();

                let progress_full_str =
                    String::from("[") + &progress_str + &padding + &String::from("]");

                println!(
                    "Progress: {} {}%",
                    progress_full_str,
                    (progress * 100.0) as usize
                );
                *total_progress.lock().unwrap() = progress;

                tx_clone3
                    .send(PartialRenderMessage::Progress {
                        0: *total_progress.lock().unwrap(),
                    })
                    .unwrap();

                if *total_progress.lock().unwrap() >= 1.0 {
                    break;
                }
            }
            Err(_) => {} // todo
        }
    });

    for bbox in bboxes {
        let scene_clone = scene.clone();
        let tx_clone2 = tx.clone();

        let progress_clone = progress.clone();

        let progtx = progtx.clone();
        pool.execute(move || {
            let pixels = render_block(scene_clone, camera, settings, bbox, progtx);
            *progress_clone.lock().unwrap() += 1;

            tx_clone2
                .send(PartialRenderMessage::PixelData {
                    0: PartialRenderMessagePixels {
                        pixel_data: Arc::new(pixels),
                        bbox: bbox,
                    },
                })
                .unwrap();
        });
    }

    pool.join();
}

fn collide<'a>(r: &Ray, scn: &Scene) -> Option<(CollisionData, Object)> {
    let mut closest_obj: Option<Object> = None;
    let mut closest_data: Option<CollisionData> = None;
    let mut closest_distance: f32 = 99999999999.9;

    for obj in scn.objects.iter() {
        match obj.geometry.hit(r) {
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
        match light.geometry.hit(r) {
            None => continue,
            Some(data) => {
                let distance = (r.origin - data.point).length();
                if distance < closest_distance {
                    closest_obj = Some(light.clone());
                    closest_data = Some(data);
                    closest_distance = distance;
                }
            }
        }
    }

    match closest_obj {
        Some(obj) => Some((closest_data.unwrap(), obj)),
        None => None,
    }
}

/**
 * We hit the scene with a ray, if it hit something then we take the objects material into
 * account how to render it but also do a shadow ray towards all sources of light to see if we should
 * light the pixel in order to produce soft shadows
 */
fn ray_color(r: &Ray, scn: &Scene, depth: i16, shadow_samples: u32) -> Color {
    if depth <= 0 {
        return Color::default();
    }

    match collide(r, &scn) {
        Some(collision_data) => {
            let _mat = collision_data.1.mat;

            match collision_data.1.kind {
                // todo: should be actual color of light?
                loader::Kind::Light => Color::white(),
                loader::Kind::Object => {
                    let collision_point = collision_data.0.point;
                    let collision_normal = collision_data.0.normal;

                    let mut color = Vec3::new(0.0, 0.0, 0.0);

                    match collision_data.1.mat {
                        material::Material::Lambertian(m) => {
                            color.x = m.color.r;
                            color.y = m.color.g;
                            color.z = m.color.b;
                        }
                        material::Material::Metal(m) => {
                            color.x = m.color.r;
                            color.y = m.color.g;
                            color.z = m.color.b;
                        }
                        material::Material::Dielectric(m) => {
                            color.x = m.color.r;
                            color.y = m.color.g;
                            color.z = m.color.b;
                        }
                    }

                    // nowe as we've hit the object in the scene, we need to determine
                    // it's relation to the light sources, it might be in the shadow or might be
                    // lit. In order to find that out we collide another ray from collision point towards
                    // all the light sources in the scene and light the pixel accordingly
                    // we do not care about light's color at the moment

                    let mut intensities: Vec<f32> = Vec::new();
                    for light in scn.lights() {
                        let mut collisions = 0;

                        let rays_cnt = shadow_samples;
                        let ll = light.clone();

                        let rays = (0..rays_cnt).into_iter().map(|_| {
                            let geom = &ll.geometry;
                            Ray::new(
                                collision_point,
                                (geom.pos() + geom.get_random_point()) - collision_point,
                            )
                        });

                        rays.for_each(|r| match collide(&r, &scn) {
                            None => {}
                            Some(shadow_coll) => match shadow_coll.1.kind {
                                // todo: check if it's the same light source
                                loader::Kind::Light => {
                                    collisions += 1;
                                }
                                loader::Kind::Object => {}
                            },
                        });

                        let n = light.geometry.pos();
                        let m = collision_normal.clone().normalize();
                        let dot = m.dot(n).clamp(0.0, 1.0) as f32;

                        let intense = collisions as f32 / rays_cnt as f32;
                        intensities.push(dot * intense);
                    }

                    let light_intensity =
                        intensities.iter().sum::<f32>() / intensities.len() as f32;

                    match collision_data.1.mat {
                        material::Material::Lambertian(m) => {
                            return (color * light_intensity * m.albedo).into();
                        }

                        material::Material::Metal(m) => {
                            let norm = collision_data.0.normal.clone().normalize();
                            let reflected_dir = reflect(&r.dir, &norm).normalize()
                                + random_point_in_circle() * m.fuzz;

                            // todo: without this metallic material produces borders with the color of whats behind
                            if norm.dot(r.dir) > -0.60 {
                                return (color * light_intensity).into();
                            }
                            let reflected_ray =
                                Ray::new(collision_data.0.point, reflected_dir.clone().normalize());

                            let rcol: Vec3 =
                                ray_color(&reflected_ray, &scn, depth - 1, shadow_samples).into();
                            return (color * light_intensity * m.albedo + rcol * m.albedo).into();
                        }
                        material::Material::Dielectric(_m) => {
                            // double cos_theta = fmin(dot(-unit_direction, rec.normal), 1.0);
                            // double sin_theta = sqrt(1.0 - cos_theta*cos_theta);

                            // bool cannot_refract = refraction_ratio * sin_theta > 1.0;
                            // vec3 direction;

                            // if (cannot_refract)
                            // direction = reflect(unit_direction, rec.normal);
                            // else
                            // direction = refract(unit_direction, rec.normal, refraction_ratio);

                            // scattered = ray(rec.p, direction);
                            todo!();
                        }
                    }

                    // for matte
                    // let target =
                    //     collision_data.0.point + collision_data.0.normal + random_point_in_circle();

                    // mirror/metal

                    // todo: hardcoded material number
                }
            }
        }
        None => Color::new(AMBIENT_R, AMBIENT_G, AMBIENT_B),
    }
}

fn get_bboxes_for(w: i32, h: i32, subdivisions: i32) -> Vec<BBox> {
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
pub fn reflect(v: &glam::Vec3, normal: &glam::Vec3) -> glam::Vec3 {
    *v - (*normal * 2.0 * v.dot(*normal))
}

#[cfg(test)]
mod tests {
    use super::get_bboxes_for;

    #[test]
    fn test_bbox_generator4() {
        // let b = getBBoxesFor(8, 8, 8);
        // assert_eq!(b.len(), 64);

        let bboxes = get_bboxes_for(4, 4, 4);
        assert_eq!(bboxes.len(), 16);
    }
}
