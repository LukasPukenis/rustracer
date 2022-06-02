use clap::Parser;

mod animation;
mod camera;
mod gui;
mod loader;
mod material;
mod ray;
mod renderer;
mod scene;
mod sphere;
mod support;
mod vec3;

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    width: u32,

    #[clap(short, long)]
    height: u32,

    #[clap(short, long)]
    output: String,

    #[clap(short, long)]
    scene: String,

    #[clap(short, long)]
    nogui: bool,
}

fn main() {
    let args = Args::parse();
    // let width = args.width;
    // let height = args.height;

    // todo: pass cli properly to everywhere
    let width = 400;
    let height = 400;
    let mut scene = scene::Scene::new(width as u64, height as u64);

    let objects = loader::load(&args.scene);
    let mut animator = animation::Animator::new();

    for obj in objects {
        match obj.2 {
            loader::Kind::Object => scene.add_object(obj.0, obj.1),
            loader::Kind::Light => scene.add_light(obj.0, obj.1),
        }
    }

    // todo: hardcoded light!
    // let light_pos = vec3::Vec3::new_with(0.0, 0.0, -1.0);
    // let light_radius = 0.1;
    // let light_sphere = sphere::Sphere::new(light_pos, light_radius);
    // let mut lightmat = material::Material::new();
    // lightmat.color.r = 255.0;
    // lightmat.color.g = 255.0;
    // lightmat.color.b = 255.0;

    // scene.add_light(Arc::new(Mutex::new(light_sphere)), lightmat);

    let mut GUI = gui::GUIApp::new(Arc::new(Mutex::new(scene)));

    if args.nogui {
        // todo
    } else {
        let system = support::init(file!());
        system.main_loop(move |_, ui, ctx, texs| {
            GUI.update(ui, ctx, texs);
        });
    }
}
