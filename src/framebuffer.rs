use raylib::core::texture::RaylibTexture2D;
use raylib::prelude::*;

pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pixels: Vec<u8>, // RGBA8, width*height*4
    texture: Texture2D,
}

impl Framebuffer {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let pixels = vec![0u8; (width * height * 4) as usize];
        let img = Image::gen_image_color(width, height, Color::BLACK);
        let texture = rl
            .load_texture_from_image(thread, &img)
            .expect("no se pudo crear la textura del framebuffer");
        Self {
            width,
            height,
            pixels,
            texture,
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.pixels[idx] = color.r;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.b;
        self.pixels[idx + 3] = 255;
    }

    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        let a = color.a as f32 / 255.0;
        self.pixels[idx] = (self.pixels[idx] as f32 * (1.0 - a) + color.r as f32 * a) as u8;
        self.pixels[idx + 1] =
            (self.pixels[idx + 1] as f32 * (1.0 - a) + color.g as f32 * a) as u8;
        self.pixels[idx + 2] =
            (self.pixels[idx + 2] as f32 * (1.0 - a) + color.b as f32 * a) as u8;
        self.pixels[idx + 3] = 255;
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        for py in y.max(0)..(y + h).min(self.height) {
            for px in x.max(0)..(x + w).min(self.width) {
                self.set_pixel(px, py, color);
            }
        }
    }

    pub fn draw_circle_filled(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r2 = radius * radius;
        for py in (cy - radius).max(0)..(cy + radius).min(self.height) {
            for px in (cx - radius).max(0)..(cx + radius).min(self.width) {
                let dx = px - cx;
                let dy = py - cy;
                if dx * dx + dy * dy <= r2 {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_circle_blend(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r2 = radius * radius;
        for py in (cy - radius).max(0)..(cy + radius).min(self.height) {
            for px in (cx - radius).max(0)..(cx + radius).min(self.width) {
                let dx = px - cx;
                let dy = py - cy;
                if dx * dx + dy * dy <= r2 {
                    self.blend_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.set_pixel(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn upload(&mut self) {
        let _ = self.texture.update_texture(&self.pixels);
    }

    /// Dibuja el framebuffer completo en pantalla: UNA sola llamada.
    pub fn present(&self, d: &mut RaylibDrawHandle) {
        let src = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);
        let dest = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);
        d.draw_texture_pro(&self.texture, src, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
    }
}