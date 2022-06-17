use crate::material;
use crate::scene::Hitable;
use crate::sphere::Sphere;
use crate::vec3::Vec3;
use serde_json::Value;
use std::fs;
use std::sync::Arc;

#[derive(PartialEq, Clone)]
pub enum Kind {
    Object,
    Light,
}

pub fn load(path: &str) -> Vec<(Arc<dyn Hitable>, material::Material, Kind)> {
    let contents = fs::read_to_string(path).expect("file not found");
    let j: Value = serde_json::from_str(&contents).unwrap();
    let mut results = Vec::new();

    for item in j.as_array().unwrap() {
        let obj = build_object_from_string(item);
        results.push(obj);
    }

    results
}

fn panic_on_range(x: f64) {
    if x < 0.0 || x > 1.0 {
        panic!("Range must be inside [0;1]")
    }
}

fn build_object_from_string(s: &Value) -> (Arc<dyn Hitable>, material::Material, Kind) {
    let mat: material::Material;

    match &s["material"] {
        m => match m {
            material => {
                let mut color = material::Color::default();
                let col = &material["color"];
                color.r = col["r"].as_f64().unwrap();
                color.g = col["g"].as_f64().unwrap();
                color.b = col["b"].as_f64().unwrap();
                panic_on_range(color.r);
                panic_on_range(color.g);
                panic_on_range(color.b);

                match material["type"].as_str().unwrap() {
                    "lambertian" => {
                        mat = material::Material::Lambertian(material::Lambertian {
                            albedo: material["albedo"].as_f64().unwrap(),
                            color,
                        });
                    }
                    "metal" => {
                        mat = material::Material::Metal(material::Metal {
                            fuzz: material["fuzz"].as_f64().unwrap(),
                            albedo: material["albedo"].as_f64().unwrap(),
                            color,
                        });
                    }
                    "dielectric" => {
                        mat = material::Material::Dielectric(material::Dielectric {
                            refraction: material["refraction"].as_f64().unwrap(),
                            color,
                        });
                    }
                    _ => panic!("material not supported"),
                }
            }
        },
    }

    let obj: Arc<dyn Hitable>;
    let kind: Kind;

    match s["type"].as_str().unwrap() {
        "sphere" => {
            obj = Arc::new(build_sphere(s));
            kind = Kind::Object;
        }
        "point_light" => {
            obj = Arc::new(build_sphere(s));
            kind = Kind::Light;
        }
        _ => panic!("unrecognized type"),
    }

    (obj, mat, kind)
}

fn build_sphere(s: &Value) -> Sphere {
    let pos = &s["pos"];
    let x = &pos["x"];
    let y = &pos["y"];
    let z = &pos["z"];

    let pos = Vec3::new_with(
        x.as_f64().unwrap(),
        y.as_f64().unwrap(),
        z.as_f64().unwrap(),
    );

    let radius = s["radius"].as_f64().unwrap();

    Sphere::new(pos, radius)
}
