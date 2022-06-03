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
    let width = args.width;
    let height = args.height;

    let mut scene = scene::Scene::new(width as u64, height as u64);

    let objects = loader::load(&args.scene);
    let mut animator = animation::Animator::new();

    for obj in objects {
        match obj.2 {
            loader::Kind::Object => scene.add_object(obj.0, obj.1),
            loader::Kind::Light => scene.add_light(obj.0, obj.1),
        }

        match obj.3 {
            Some(anim) => animator.add(anim),
            None => {}
        }
    }

    let mut GUI = Arc::new(Mutex::new(gui::GUIApp::new(
        Arc::new(Mutex::new(scene)),
        width,
        height,
        animator,
    )));

    if args.nogui {
        // todo
    } else {
        let system = support::init(file!());
        let gui_clone = GUI.clone();
        system.main_loop(move |_, ui, ctx, texs| {
            gui_clone.lock().unwrap().update(ui, ctx, texs);
        });
    }
}
