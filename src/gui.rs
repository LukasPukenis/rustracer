const AA_SAMPLES_MIN: u32 = 1;
const AA_SAMPLES_MAX: u32 = 32;
const MIN_POS: f64 = -8.0;
const MAX_POS: f64 = 8.0;

use crate::animation::Animator;
use crate::renderer::*;
use crate::scene::Scene;
use std::borrow::Cow;

use std::time::SystemTime;

use chrono::{Datelike, Timelike, Utc};

use std::rc::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::camera;
use crate::scene;
use crate::vec3;

use glium::{
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Texture2d,
};
use imgui;
use imgui::Condition;
use imgui::Image;
use imgui::ProgressBar;
use imgui::TextureId;
use imgui::Textures;
use imgui::Ui;
use imgui_glium_renderer::Texture;

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Copy, Clone)]
enum State {
    Idle,
    Rendering(f32),
    Animating,
}

// pub struct RenderMessage {
//     pixel_data: Arc<Mutex<Vec<scene::Pixel>>>,
//     time: std::time::Duration,
// }

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
    pixel_data: Arc<Mutex<Vec<scene::Pixel>>>,
    bbox: BBox,
    progress: f32,
}

impl PartialRenderMessage {
    pub fn new(
        pixel_data: Arc<Mutex<Vec<scene::Pixel>>>,
        bbox: BBox,
        progress: f32,
    ) -> PartialRenderMessage {
        assert_eq!(pixel_data.lock().unwrap().len(), (bbox.w * bbox.h) as usize);

        PartialRenderMessage {
            pixel_data,
            bbox,
            progress,
        }
    }
}

pub struct GUIApp {
    animator: Animator,
    settings: Settings,
    frame: u32,
    state: State,
    last_state: State,
    texture_id: Option<TextureId>,
    scene: Arc<Mutex<Scene>>,
    renderer: Renderer,
    camera: camera::Camera,
    // pixel_channel: (Sender<RenderMessage>, Receiver<RenderMessage>),
    pixel_channel: (Sender<PartialRenderMessage>, Receiver<PartialRenderMessage>),
    progress_channel: (Sender<f32>, Receiver<f32>),
    pixels: Vec<PartialRenderMessage>,
}

#[derive(Copy, Clone)]
pub struct Settings {
    pub samples: u32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { samples: 1 }
    }
}

impl GUIApp {
    pub fn new(scene: Arc<Mutex<Scene>>, width: u32, height: u32, animator: Animator) -> GUIApp {
        let (tx, rx): (Sender<PartialRenderMessage>, Receiver<PartialRenderMessage>) =
            mpsc::channel();

        let (progtx, progrx) = mpsc::channel();

        GUIApp {
            animator: animator,
            scene: scene,
            state: State::Idle,
            last_state: State::Idle,
            camera: camera::Camera::new(
                vec3::Vec3::new_with(0.0, 0.0, 1.0), // pos
                vec3::Vec3::new_with(0.0, 0.0, 1.0), // dir
                60.0,
            ),
            pixels: Vec::new(),
            texture_id: None,
            frame: 0,
            renderer: Renderer::new(width, height), // todo: dims
            pixel_channel: (tx, rx),
            progress_channel: (progtx, progrx),
            settings: Settings::default(),
        }
    }

    fn advance_frame(&mut self) {
        let fps = 60.0;
        let time = (1000.0 / fps * 0.01) * self.frame as f64;

        self.frame += 1;
        self.animator.update(time);
        self.render();
    }

    pub fn update(
        &mut self,
        ui: &Ui,
        ctx: &Rc<glium::backend::Context>,
        texs: &mut Textures<imgui_glium_renderer::Texture>,
    ) {
        ui.window("Renderer")
            .size([500.0, 500.0], Condition::FirstUseEver)
            .resizable(false)
            .position([0.0, 0.0], Condition::Always)
            .movable(false)
            .build(|| {
                if let Some(texture_id) = self.texture_id {
                    Image::new(texture_id, [300.0, 300.0]).build(ui);
                }

                // todo: pass CLI path
                self.show_save_button(ui, None);

                match self.state {
                    State::Idle => {
                        if ui.button("Render") {
                            // todo: ugh
                            self.last_state = State::Idle;
                            self.render();
                        } else if ui.button("Animate") {
                            self.last_state = State::Animating;
                            self.render();
                        }
                    }
                    State::Animating => {
                        if self.frame > 200 {
                            self.frame = 0;
                            self.state = State::Idle;
                        } else {
                            self.advance_frame();
                        }
                    }

                    State::Rendering(progress) => {
                        // todo: suboptimal, we spam the channel with data
                        loop {
                            match self.progress_channel.1.try_recv() {
                                Ok(data) => self.state = State::Rendering(data),
                                Err(_e) => break,
                            }
                        }

                        // todo: this doesnt work with the new partialmessage model
                        ProgressBar::new(progress)
                            .overlay_text("Rendering...")
                            .build(&ui);

                        match self.pixel_channel.1.try_recv() {
                            Ok(data) => {
                                let pixels = data.pixel_data.lock().unwrap();

                                for pixel in &*pixels {
                                    let (x, y, color) = (pixel.x, pixel.y, pixel.color);
                                    // todo: bbox
                                    self.renderer.putpixel(
                                        (x) as u32,
                                        (y) as u32,
                                        // todo: maybe bbox doesnt need w and h?
                                        color,
                                    );
                                }

                                let (width, height, pixels) = self.renderer.data();

                                // todo: what is cow
                                let raw = RawImage2d {
                                    data: Cow::Owned(Vec::from(pixels)),
                                    width: width as u32,
                                    height: height as u32,
                                    format: ClientFormat::U8U8U8,
                                };

                                // // todo: question mark?
                                let gl_texture = Texture2d::new(ctx, raw).unwrap();
                                let texture = Texture {
                                    texture: Rc::new(gl_texture),
                                    sampler: SamplerBehavior {
                                        magnify_filter: MagnifySamplerFilter::Linear,
                                        minify_filter: MinifySamplerFilter::Linear,
                                        ..Default::default()
                                    },
                                };

                                match self.texture_id {
                                    Some(t) => {
                                        texs.replace(t, texture);
                                    }
                                    None => {
                                        self.texture_id = Some(texs.insert(texture));
                                    }
                                }

                                println!("progress {}", data.progress);
                                if data.progress >= 1.0 {
                                    self.state = self.last_state;
                                }
                            }
                            Err(e) => {}
                        }
                    }
                }
            });

        ui.window("Options")
            .size([400.0, 600.0], Condition::FirstUseEver)
            .resizable(false)
            .position([510.0, 0.0], Condition::Always)
            .movable(false)
            .build(|| {
                ui.text("Animations");
                let fps = 60.0;
                let time = (1000.0 / fps * 0.01) * self.frame as f64;
                if ui.button("Frame") {
                    self.advance_frame();
                }
                ui.text(format!(
                    "fps: {} frame: {} time: {:.2}",
                    fps, self.frame, time
                ));

                ui.separator();
                ui.text("Misc");
                ui.slider_config("AA Samples", AA_SAMPLES_MIN, AA_SAMPLES_MAX)
                    .build(&mut self.settings.samples);

                let k = 50.0;
                ui.text("Camera");

                ui.text("Direction");
                ui.slider_config("xx", -k, k)
                    .display_format("%.01f")
                    .build(&mut self.camera.dir.x);
                ui.slider_config("yu", -k, k)
                    .display_format("%.01f")
                    .build(&mut self.camera.dir.y);
                ui.slider_config("zz", -k, k)
                    .display_format("%.01f")
                    .build(&mut self.camera.dir.z);

                ui.text("Position");
                ui.slider_config("x", -k, k)
                    .display_format("%.01f")
                    .build(&mut self.camera.pos.x);
                ui.slider_config("y", -k, k)
                    .display_format("%.01f")
                    .build(&mut self.camera.pos.y);
                ui.slider_config("z", -k, k)
                    .display_format("%.01f")
                    .build(&mut self.camera.pos.z);
                ui.slider_config("fov", 1.0, 200.0)
                    .build(&mut self.camera.fov);
                ui.separator();

                {
                    ui.text("Objects");
                    let mut obj_id = 0;
                    if let Some(_t) = ui.tree_node("Objects") {
                        for obj in self.scene.lock().unwrap().objects() {
                            ui.text(format!("obj({})", obj_id));
                            obj_id += 1;
                            let stack = ui.push_id(format!("obj({})", obj_id));
                            let mut g = obj.geometry.lock().unwrap();
                            let pos = g.pos_mut();

                            // todo: slides all sliders for all objects
                            ui.slider_config("x", MIN_POS, MAX_POS)
                                .display_format("%.01f")
                                .build(&mut pos.x);
                            ui.slider_config("y", MIN_POS, MAX_POS)
                                .display_format("%.01f")
                                .build(&mut pos.y);
                            ui.slider_config("z", MIN_POS, MAX_POS)
                                .display_format("%.01f")
                                .build(&mut pos.z);
                            stack.pop();
                        }
                    }
                }

                {
                    ui.text("Lights");
                    let mut obj_id = 0;
                    if let Some(_t) = ui.tree_node("Lights") {
                        for obj in self.scene.lock().unwrap().lights() {
                            ui.text(format!("light({})", obj_id));
                            let stack = ui.push_id(format!("light({})", obj_id));
                            obj_id += 1;
                            let mut g = obj.geometry.lock().unwrap();
                            let pos = g.pos_mut();

                            // todo: slides all sliders for all lights
                            ui.slider_config("x", MIN_POS, MAX_POS)
                                .display_format("%.01f")
                                .build(&mut pos.x);
                            ui.slider_config("y", MIN_POS, MAX_POS)
                                .display_format("%.01f")
                                .build(&mut pos.y);
                            ui.slider_config("z", MIN_POS, MAX_POS)
                                .display_format("%.01f")
                                .build(&mut pos.z);
                            stack.pop();
                        }
                    }
                }
            });
    }

    fn show_save_button(&self, ui: &Ui, path: Option<&str>) {
        if self.texture_id.is_some() {
            let save_path: String;

            match path {
                Some(p) => save_path = String::from(p),
                None => {
                    let now = Utc::now();
                    let hour = now.hour();
                    save_path = format!(
                        "render_{}-{}-{}_{}-{}-{}.png",
                        now.year(),
                        now.month(),
                        now.day(),
                        hour,
                        now.minute(),
                        now.second()
                    );
                }
            }

            if ui.button("Save PNG") {
                self.renderer.save(&save_path);
                println!("Saved at '{}'", save_path);
            }
        }
    }

    fn render(&mut self) {
        self.state = State::Rendering(0.0);
        self.renderer.clear();
        self.pixels.clear();

        let _tx2 = self.pixel_channel.0.clone();
        println!("Spawning a thread...");

        // todo: this moves in px2 into thread, just note
        let px2 = self.progress_channel.0.clone();

        let rf = self.scene.clone();
        let settings = self.settings;

        let cam = self.camera;
        let chan = self.pixel_channel.0.clone();

        let _h = thread::spawn(move || {
            let start = SystemTime::now();
            let _pixels = scene::draw(rf, cam, 1, px2, settings, chan);
            let _elapsed = start.elapsed().unwrap();

            // tx2.send(RenderMessage {
            //     pixel_data: (Arc::new(Mutex::new(pixels))),
            //     time: (elapsed),
            // })
        });
    }
}
