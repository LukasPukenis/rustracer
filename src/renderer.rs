use crate::material::Color;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Renderer {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Renderer {
        let len = (width * height * 4) as usize;
        let mut data = Vec::with_capacity(len);
        for _i in 0..len as usize {
            data.push(0);
        }

        Renderer {
            width,
            height,
            buffer: data,
        }
    }

    pub fn putpixel(&mut self, x: u32, y: u32, color: Color) {
        let base = ((y * self.width + x) * 4) as usize;

        fn gamma(i: f64) -> f64 {
            i.sqrt()
        }

        self.buffer[base + 0] = (gamma(color.r) * 255.0) as u8;
        self.buffer[base + 1] = (gamma(color.g) * 255.0) as u8;
        self.buffer[base + 2] = (gamma(color.b) * 255.0) as u8;
        self.buffer[base + 3] = 255;
    }

    pub fn save(&self, path: &str) {
        let path = Path::new(path);
        let file = File::create(path).unwrap();
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_trns(vec![0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]);
        encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2)); // 1.0 / 2.2, unscaled, but rounded
        let source_chromaticities = png::SourceChromaticities::new(
            // Using unscaled instantiation here
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000),
        );
        encoder.set_source_chromaticities(source_chromaticities);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&self.buffer).unwrap();
    }
}
