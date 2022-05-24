const AA_SAMPLES_MIN: u8 = 1;
const AA_SAMPLES_MAX: u8 = 16;
const MIN_POS: f64 = -8.0;
const MAX_POS: f64 = 8.0;

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

enum State {
    Idle,
    Rendering(f32)
}

pub struct RenderMessage {
    pixel_data: Arc<Mutex<Vec<scene::Pixel>>>,
    time: std::time::Duration,
}

pub struct GUIApp {
    settings: Settings,
    state: State,
    texture_id: Option<TextureId>,
    scene: Arc<Mutex<Scene>>,
    renderer: Renderer,
    camera: camera::Camera,
    pixel_channel: (
        Sender<RenderMessage>,
        Receiver<RenderMessage>,
    ),
    progress_channel: (Sender<f32>, Receiver<f32>),
}

#[derive(Copy, Clone)]
pub struct Settings {
    pub samples: u8
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            samples: 1,
        }
    }
}

impl GUIApp {
    pub fn new(scene: Arc<Mutex<Scene>>) -> GUIApp {
        let (tx, rx): (
            Sender<RenderMessage>,
            Receiver<RenderMessage>,
        ) = mpsc::channel();

        let (progtx, progrx) = mpsc::channel();

        GUIApp {
            scene: scene,
            state: State::Idle,
            camera: camera::Camera::new(vec3::Vec3::new_with(0.0, 0.0, 0.0),
                                        vec3::Vec3::new_with(0.0, 0.0, 1.0),
                                        46.8),
            texture_id: None,
            renderer: Renderer::new(400, 400), // todo: dims
            pixel_channel: (tx, rx),
            progress_channel: (progtx, progrx),
            settings: Settings::default(),
        }
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
                    Image::new(texture_id, [400.0, 400.0]).build(ui);
                }

                // todo: pass CLI path
                self.show_save_button(ui, None);

                match self.state {
                    State::Idle => {
                        if ui.button("Render") {
                            self.state = State::Rendering(0.0);
                            self.renderer.clear();

                            let tx2 = self.pixel_channel.0.clone();
                            println!("Spawning a thread...");

                            // todo: this moves in px2 into thread, just note
                            let px2 = self.progress_channel.0.clone();

                            let rf = self.scene.clone();
                            let settings = self.settings;

                            let cam = self.camera;
                            let _h = thread::spawn(move || {
                                println!("Inside of a thread");
                                let start = SystemTime::now();
                                let pixels = scene::draw(rf, &cam, 1, px2, settings);
                                let elapsed = start.elapsed().unwrap();

                                tx2.send(RenderMessage { pixel_data: (Arc::new(Mutex::new(pixels))), time: (elapsed) })
                            });

                            // self.render_handle = Some(h);
                            // h.join();
                            // render_handle.join().unwrap();
                        }
                    },
                    State::Rendering(progress) => {
                        // todo: suboptimal, we spam the channel with data
                        loop {
                            match self.progress_channel.1.try_recv() {
                                Ok(data) => self.state = State::Rendering(data),
                                Err(_e) => break,
                            }
                        }

                        ProgressBar::new(progress)
                            .overlay_text("Rendering...")
                            .build(&ui);

                        match self.pixel_channel.1.try_recv() {
                            Ok(data) => {
                                let pixels = data.pixel_data.lock().unwrap();

                                // todo: why dereferenced it works, but referneced doesn't
                                for pixel in &*pixels {
                                    let (x, y, color) = (pixel.x, pixel.y, pixel.color);
                                    self.renderer.putpixel(x as u32, y as u32, color);
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
                                    },
                                    None => {
                                        self.texture_id = Some(texs.insert(texture));
                                    }
                                }

                                self.state = State::Idle;
                            }
                            Err(_e) => {
                                return;
                            }
                        }
                    },
                }
            });

    ui.window("Tree").size([400.0, 600.0], Condition::FirstUseEver)
        .resizable(false)
        .position([510.0, 0.0], Condition::Always)
        .movable(false)
        .build(|| {
            ui.text("Misc");
            ui.slider_config("AA Samples", AA_SAMPLES_MIN, AA_SAMPLES_MAX).build(&mut self.settings.samples);

            ui.text("Camera");
            ui.slider_config("x", -8.0, 8.0).display_format("%.01f").build(&mut self.camera.pos.x);
            ui.slider_config("y", -8.0, 8.0).display_format("%.01f").build(&mut self.camera.pos.y);
            ui.slider_config("z", -8.0, 8.0).display_format("%.01f").build(&mut self.camera.pos.z);
            ui.slider_config("fov", 1.0, 200.0).build(&mut self.camera.fov);
            ui.separator();

            {
                ui.text("Objects");
                println!("----------");
                let mut obj_id = 0;
                if let Some(_t) = ui.tree_node("Objects") {
                    for obj in self.scene.lock().unwrap().objects() {
                        ui.text(format!("obj({})", obj_id));
                        obj_id +=1 ;
                        let stack = ui.push_id(format!("obj({})", obj_id));
                        let mut g = obj.geometry.lock().unwrap();
                        let pos = g.pos_mut();

                        // todo: slides all sliders for all objects
                        ui.slider_config("x", MIN_POS, MAX_POS).display_format("%.01f").build(&mut pos.x);
                        ui.slider_config("y", MIN_POS, MAX_POS).display_format("%.01f").build(&mut pos.y);
                        ui.slider_config("z", MIN_POS, MAX_POS).display_format("%.01f").build(&mut pos.z);
                        stack.pop();
                    }
                }
                println!("----------");
            }

            {
                ui.text("Lights");
                let mut obj_id = 0;
                if let Some(_t) = ui.tree_node("Lights") {
                    for obj in self.scene.lock().unwrap().lights() {
                        ui.text(format!("light({})", obj_id));
                        let stack = ui.push_id(format!("light({})", obj_id));
                        obj_id +=1 ;
                        let mut g = obj.geometry.lock().unwrap();
                        let pos = g.pos_mut();

                        // todo: slides all sliders for all lights
                        ui.slider_config("x", MIN_POS, MAX_POS).display_format("%.01f").build(&mut pos.x);
                        ui.slider_config("y", MIN_POS, MAX_POS).display_format("%.01f").build(&mut pos.y);
                        ui.slider_config("z", MIN_POS, MAX_POS).display_format("%.01f").build(&mut pos.z);
                        stack.pop();
                    }
                }
            }
        });

    }

    fn show_save_button(&self, ui: &Ui, path: Option<&str>) {
        if self.texture_id.is_some() {
            let mut save_path: String;

            match path {
                Some(p) => save_path = String::from(p),
                None => {
                    let now = Utc::now();
                    let hour = now.hour();
                    save_path = format!("render_{}-{}-{}_{}-{}-{}.png", now.year(), now.month(), now.day(), hour, now.minute(), now.second());
                }
            }

            if ui.button("Save PNG") {
                self.renderer.save(&save_path);
                println!("Saved at '{}'", save_path);
            }
        }
    }
}
