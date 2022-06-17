use crate::scene::Scene;

use crate::camera;
use crate::renderer;
use crate::scene;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

use std::sync::Arc;
use std::thread;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct BBox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub enum PartialRenderMessage {
    PixelData(PartialRenderMessagePixels),
    Progress(f64),
}
pub struct PartialRenderMessagePixels {
    pub pixel_data: Arc<Vec<scene::Pixel>>,
    pub bbox: BBox,
}

#[derive(Copy, Clone)]
pub struct Settings {
    // samples per pixel
    pub samples: u32,
    // threads
    pub threads: usize,

    // how many blocks to split the scene to. ideally should map to N*threads but some parts might be more complex
    // so it's totally guesswork at the moment
    pub bboxes: usize,

    // soft shadows are produced by throwing rays into the light source and averaging how many hit it
    // the more rays - the better quality of a shadow
    pub shadow_samples: u32,
}

impl Settings {
    pub fn new(samples: u32, threads: usize, bboxes: usize, shadow_samples: u32) -> Settings {
        Settings {
            samples,
            threads,
            bboxes,
            shadow_samples,
        }
    }
}
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            samples: 1,
            threads: 1,
            bboxes: 1,
            shadow_samples: 1,
        }
    }
}

pub fn render(
    renderer: Arc<Mutex<renderer::Renderer>>,
    camera: camera::Camera,
    scene: Arc<Scene>,
    _width: u32,
    _height: u32,
    settings: Settings,
) {
    let (tx, rx): (Sender<PartialRenderMessage>, Receiver<PartialRenderMessage>) = mpsc::channel();

    let h = thread::spawn(move || loop {
        match rx.try_recv() {
            Ok(data) => match data {
                PartialRenderMessage::Progress(progress) => {
                    if progress >= 1.0 {
                        break;
                    }
                }
                PartialRenderMessage::PixelData(data) => {
                    let mut locked_renderer = renderer.lock().unwrap();

                    for pixel in &*data.pixel_data {
                        locked_renderer.putpixel(pixel.x as u32, pixel.y as u32, pixel.color);
                    }
                }
            },
            Err(_e) => {
                thread::yield_now();
            }
        }
    });

    scene::draw(scene, camera, settings, tx);

    h.join().unwrap();
}
