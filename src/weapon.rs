// weapon.rs
use raylib::prelude::*;

pub struct Weapon {
    pub texture: Texture2D,
    bob_phase: f32,
    recoil: f32,
    muzzle_flash_timer: f32,
}

impl Weapon {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let img = crate::textures::generate_pistol_image();
        Self {
            texture: rl.load_texture_from_image(thread, &img).expect("pistol tex"),
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

    pub fn draw(&self, d: &mut RaylibDrawHandle, screen_w: i32, screen_h: i32) {
        let scale = 1.6; // antes 4.5, ocupaba demasiada pantalla
        let tex_w = self.texture.width as f32 * scale;
        let tex_h = self.texture.height as f32 * scale;

        let bob_x = self.bob_phase.sin() * 8.0;
        let bob_y = (self.bob_phase * 2.0).sin().abs() * 6.0;

        let base_x = screen_w as f32 / 2.0 - tex_w / 2.0 + bob_x;
        let base_y = screen_h as f32 - tex_h * 0.85 + bob_y;
        let recoil_offset_y = self.recoil * 14.0;

        let dest = Rectangle::new(base_x, base_y + recoil_offset_y, tex_w, tex_h);
        let src = Rectangle::new(0.0, 0.0, self.texture.width as f32, self.texture.height as f32);
        d.draw_texture_pro(&self.texture, src, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);

        if self.muzzle_flash_timer > 0.0 {
            let flash_x = screen_w as f32 / 2.0 + bob_x;
            let flash_y = base_y + recoil_offset_y - 6.0 * scale;
            let alpha = (self.muzzle_flash_timer / 0.06 * 255.0) as u8;
            d.draw_circle(
                flash_x as i32,
                flash_y as i32,
                14.0,
                Color::new(255, 230, 120, alpha),
            );
            d.draw_circle(
                flash_x as i32,
                flash_y as i32,
                7.0,
                Color::new(255, 255, 220, alpha),
            );
        }
    }
}

pub fn draw_crosshair(d: &mut RaylibDrawHandle, screen_w: i32, screen_h: i32) {
    let cx = screen_w / 2;
    let cy = screen_h / 2;
    let size = 8;
    let gap = 3;
    let color = Color::new(255, 255, 255, 220);
    d.draw_line(cx - size - gap, cy, cx - gap, cy, color);
    d.draw_line(cx + gap, cy, cx + size + gap, cy, color);
    d.draw_line(cx, cy - size - gap, cx, cy - gap, color);
    d.draw_line(cx, cy + gap, cx, cy + size + gap, color);
}