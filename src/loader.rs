use crate::animation::Animation;
use crate::animation::AnimationProperty;
use crate::animation::Easing;
use crate::material::Material;
use crate::scene::Hitable;
use crate::sphere::Sphere;
use crate::vec3::Vec3;
use serde_json::Value;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(PartialEq, Clone)]
pub enum Kind {
    Object,
    Light,
}

pub fn load(path: &str) -> Vec<(Arc<Mutex<dyn Hitable>>, Material, Kind, Option<Animation>)> {
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

fn parse_ease(s: &str) -> Easing {
    match s {
        "linear" => Easing::LINEAR,
        _ => todo!(),
    }
}

fn parse_property(s: &str) -> AnimationProperty {
    match s {
        "x" => AnimationProperty::X,
        "y" => AnimationProperty::Y,
        "z" => AnimationProperty::Z,
        "radius" => AnimationProperty::RADIUS,
        _ => todo!(),
    }
}

// todo: Hitable is a combination and should be used split
fn build_object_from_string(
    s: &Value,
) -> (Arc<Mutex<dyn Hitable>>, Material, Kind, Option<Animation>) {
    let mut mat = Material::new();

    match &s["material"] {
        m => match m {
            material => {
                mat.reflective = material["reflective"].as_f64().unwrap();

                let col = &material["color"];
                mat.color.r = col["r"].as_f64().unwrap();
                mat.color.g = col["g"].as_f64().unwrap();
                mat.color.b = col["b"].as_f64().unwrap();
                panic_on_range(mat.color.r);
                panic_on_range(mat.color.g);
                panic_on_range(mat.color.b);
            }
        },
    }

    let obj: Arc<Mutex<dyn Hitable>>;
    let kind: Kind;

    match s["type"].as_str().unwrap() {
        "sphere" => {
            obj = Arc::new(Mutex::new(build_sphere(s)));
            kind = Kind::Object;
        }
        "point_light" => {
            obj = Arc::new(Mutex::new(build_sphere(s)));
            kind = Kind::Light;
        }
        _ => panic!("unrecognized type"),
    }

    let animation: Option<Animation> = None;
    match &s["animations"] {
        _anim => {
            for a in s["animations"].as_array().unwrap() {
                for prop in a.as_object() {
                    let _anim = Animation::new(
                        obj.clone(),
                        parse_property(prop["prop"].as_str().unwrap()),
                        prop["from"].as_f64().unwrap(),
                        prop["to"].as_f64().unwrap(),
                        prop["time"].as_f64().unwrap(),
                        parse_ease(prop["ease"].as_str().unwrap()),
                    );
                }
            }
        }
        _ => {}
    }

    (obj, mat, kind, animation)
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
