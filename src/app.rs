const AA_SAMPLES_MIN: u32 = 1;
const AA_SAMPLES_MAX: u32 = 32;
const MIN_POS: f64 = -8.0;
const MAX_POS: f64 = 8.0;

use crate::scene::Scene;

use crate::camera;
use crate::renderer;
use crate::scene;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use std::sync::Arc;
use std::sync::Mutex;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct BBox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl BBox {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> BBox {
        BBox { x, y, w, h }
    }
}

pub struct PartialRenderMessage {
    pixel_data: Arc<Vec<scene::Pixel>>,
    bbox: BBox,
    progress: f32,
}

impl PartialRenderMessage {
    pub fn new(
        pixel_data: Arc<Vec<scene::Pixel>>,
        bbox: BBox,
        progress: f32,
    ) -> PartialRenderMessage {
        assert_eq!(pixel_data.len(), (bbox.w * bbox.h) as usize);

        PartialRenderMessage {
            pixel_data,
            bbox,
            progress,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Settings {
    // samples per pixel
    pub samples: u32,
    // threads
    pub threads: usize,
    // how many blocks to split the scene to. ideally should map to N*threads but parts might be more complex
    // for some thread. Todo: do something for that
    pub bboxes: usize,
}

impl Settings {
    pub fn new(samples: u32, threads: usize, bboxes: usize) -> Settings {
        Settings {
            samples,
            threads,
            bboxes,
        }
    }
}
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            samples: 1,
            threads: 1,
            bboxes: 1,
        }
    }
}

pub fn render(
    renderer: &mut renderer::Renderer,
    camera: camera::Camera,
    scene: Arc<Scene>,
    _width: u32,
    _height: u32,
    settings: Settings,
) {
    let (tx, rx): (Sender<PartialRenderMessage>, Receiver<PartialRenderMessage>) = mpsc::channel();
    let (progtx, _progrx) = mpsc::channel();

    let _pixels = scene::draw(scene, camera, progtx, settings, tx);
    loop {
        match rx.try_recv() {
            Ok(data) => {
                // todo: locking
                for pixel in &*data.pixel_data {
                    renderer.putpixel(pixel.x as u32, pixel.y as u32, pixel.color);
                }
                println!("progress: {}", data.progress);
                if data.progress >= 1.0 {
                    break;
                }
            }
            Err(_e) => {}
        }
    }
}
