use crate::framebuffer::Framebuffer;
use crate::textures::PixelImage;
use raylib::prelude::Color;

pub struct Weapon {
    pixels: PixelImage,
    bob_phase: f32,
    recoil: f32,
    muzzle_flash_timer: f32,
}

impl Weapon {
    pub fn new() -> Self {
        let mut img = crate::textures::generate_pistol_image();
        Self {
            pixels: PixelImage::from_image(&mut img),
            bob_phase: 0.0,
            recoil: 0.0,
            muzzle_flash_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, is_moving: bool) {
        if is_moving {
            self.bob_phase += dt * 9.0;
        } else {
            let target = (self.bob_phase / (std::f32::consts::PI * 2.0)).round()
                * std::f32::consts::PI
                * 2.0;
            self.bob_phase += (target - self.bob_phase) * (dt * 6.0).min(1.0);
        }

        if self.recoil > 0.0 {
            self.recoil = (self.recoil - dt * 6.0).max(0.0);
        }
        if self.muzzle_flash_timer > 0.0 {
            self.muzzle_flash_timer = (self.muzzle_flash_timer - dt).max(0.0);
        }
    }

    pub fn trigger_shot(&mut self) {
        self.recoil = 1.0;
        self.muzzle_flash_timer = 0.06;
    }

    pub fn draw_fb(&self, fb: &mut Framebuffer) {
        let scale = 1.6;
        let dest_w = (self.pixels.width as f32 * scale) as i32;
        let dest_h = (self.pixels.height as f32 * scale) as i32;

        let bob_x = (self.bob_phase.sin() * 8.0) as i32;
        let bob_y = ((self.bob_phase * 2.0).sin().abs() * 6.0) as i32;

        let base_x = fb.width / 2 - dest_w / 2 + bob_x;
        let base_y = fb.height - (dest_h as f32 * 0.85) as i32 + bob_y;
        let recoil_offset_y = (self.recoil * 14.0) as i32;

        for dy in 0..dest_h {
            let sy = ((dy as f32 / scale) as i32).clamp(0, self.pixels.height - 1);
            for dx in 0..dest_w {
                let sx = ((dx as f32 / scale) as i32).clamp(0, self.pixels.width - 1);
                let c = self.pixels.get(sx, sy);
                if c.a == 0 {
                    continue;
                }
                fb.set_pixel(base_x + dx, base_y + recoil_offset_y + dy, c);
            }
        }

        if self.muzzle_flash_timer > 0.0 {
            let flash_x = fb.width / 2 + bob_x;
            let flash_y = base_y + recoil_offset_y - (6.0 * scale) as i32;
            let alpha = (self.muzzle_flash_timer / 0.06 * 255.0) as u8;
            fb.draw_circle_blend(flash_x, flash_y, 14, Color::new(255, 230, 120, alpha));
            fb.draw_circle_blend(flash_x, flash_y, 7, Color::new(255, 255, 220, alpha));
        }
    }
}

pub fn draw_crosshair_fb(fb: &mut Framebuffer) {
    let cx = fb.width / 2;
    let cy = fb.height / 2;
    let size = 8;
    let gap = 3;
    let color = Color::new(255, 255, 255, 255);
    fb.draw_line(cx - size - gap, cy, cx - gap, cy, color);
    fb.draw_line(cx + gap, cy, cx + size + gap, cy, color);
    fb.draw_line(cx, cy - size - gap, cx, cy - gap, color);
    fb.draw_line(cx, cy + gap, cx, cy + size + gap, color);
}