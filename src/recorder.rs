use gif::{Encoder, Frame, Repeat};
use raylib::prelude::*;
use std::fs::File;

pub struct GifRecorder {
    encoder: Option<Encoder<File>>,
    width: u16,
    height: u16,
    capture_interval: f32,
    timer: f32,
    frames_captured: u32,
    max_frames: u32,
}

impl GifRecorder {
    pub fn start(path: &str, width: i32, height: i32, capture_fps: f32, duration_secs: f32) -> Option<Self> {
        let file = File::create(path).ok()?;
        let mut encoder = Encoder::new(file, width as u16, height as u16, &[]).ok()?;
        encoder.set_repeat(Repeat::Infinite).ok()?;
        Some(Self {
            encoder: Some(encoder),
            width: width as u16,
            height: height as u16,
            capture_interval: 1.0 / capture_fps,
            timer: 0.0,
            frames_captured: 0,
            max_frames: (capture_fps * duration_secs).round() as u32,
        })
    }

    pub fn is_active(&self) -> bool {
        self.encoder.is_some()
    }

    pub fn update(&mut self, dt: f32, rl: &mut RaylibHandle, thread: &RaylibThread) {
        if self.encoder.is_none() {
            return;
        }
        self.timer += dt;
        if self.timer < self.capture_interval {
            return;
        }
        self.timer = 0.0;

        let image = rl.load_image_from_screen(thread);
        let colors = image.get_image_data();
        let mut rgba: Vec<u8> = Vec::with_capacity(colors.len() * 4);
        for c in colors.iter() {
            rgba.push(c.r);
            rgba.push(c.g);
            rgba.push(c.b);
            rgba.push(c.a);
        }

        if let Some(enc) = self.encoder.as_mut() {
            let mut frame = Frame::from_rgba_speed(self.width, self.height, &mut rgba, 10);
            frame.delay = (self.capture_interval * 100.0).round() as u16;
            let _ = enc.write_frame(&frame);
        }

        self.frames_captured += 1;
        if self.frames_captured >= self.max_frames {
            self.encoder = None; // al soltar el Encoder se finaliza y cierra el archivo
            println!("GIF de demostración guardado.");
        }
    }
}