use clap::Parser;

mod app;
mod camera;
mod loader;
mod material;
mod ray;
mod renderer;
mod scene;
mod sphere;

use glam::Vec3;
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

    #[clap(short, long)]
    per_pixel_samples: u32,

    #[clap(short, long)]
    shadow_samples: u32,

    #[clap(short, long)]
    threads: usize,
}

fn main() {
    let args = Args::parse();
    let width = args.width;
    let height = args.height;

    let mut scene = scene::Scene::new(width as u64, height as u64);

    let (objects, camera) = loader::load(&args.scene);

    for obj in objects {
        match obj.2 {
            loader::Kind::Object => scene.add_object(obj.0, obj.1),
            loader::Kind::Light => scene.add_light(obj.0, obj.1),
        }
    }

    let start = SystemTime::now();
    let renderer = Arc::new(Mutex::new(renderer::Renderer::new(width, height)));

    // let settings = app::Settings::new(1, 4, 16);
    let settings = app::Settings::new(
        args.per_pixel_samples,
        args.threads,
        8, // todo:
        args.shadow_samples,
    );

    app::render(
        renderer.clone(),
        camera,
        Arc::new(scene),
        width,
        height,
        settings,
    );
    let elapsed = start.elapsed().unwrap();

    println!("Rendering took {}ms", elapsed.as_millis());
    renderer.lock().unwrap().save(&args.output);
    println!("Saved at {}", args.output);
}
