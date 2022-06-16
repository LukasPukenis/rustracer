use clap::Parser;

mod app;
mod camera;
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
use std::time::SystemTime;

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
}

fn main() {
    let args = Args::parse();
    let width = args.width;
    let height = args.height;

    let mut scene = scene::Scene::new(width as u64, height as u64);

    let objects = loader::load(&args.scene);

    for obj in objects {
        match obj.2 {
            loader::Kind::Object => scene.add_object(obj.0, obj.1),
            loader::Kind::Light => scene.add_light(obj.0, obj.1),
        }
    }

    let start = SystemTime::now();
    let mut renderer = renderer::Renderer::new(width, height);
    let camera = camera::Camera::new(
        vec3::Vec3::new_with(0.0, 0.0, 1.0), // pos
        vec3::Vec3::new_with(0.0, 0.0, 1.0), // dir
        60.0,
    );

    // let settings = app::Settings::new(1, 4, 16);
    let settings = app::Settings::new(32, 8, 16);

    app::render(
        &mut renderer,
        camera,
        Arc::new(scene),
        width,
        height,
        settings,
    );
    let elapsed = start.elapsed().unwrap();

    println!("Rendering took {}ms", elapsed.as_millis());
    renderer.save(&args.output);
    println!("Saved at {}", args.output);
}
