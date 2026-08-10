use crate::player::Player;
use crate::raycasting::FOV;
use crate::textures::CreatureTextures;
use raylib::prelude::*;

const FRAME_COUNT: usize = 4;

pub struct AnimatedSprite {
    pub x: f32,
    pub y: f32,
    pub frame_index: usize,
    pub frame_timer: f32,
    pub frame_duration: f32,
    pub alive: bool,
}

impl AnimatedSprite {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            frame_index: 0,
            frame_timer: 0.0,
            frame_duration: 0.28,
            alive: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.frame_timer += dt;
        if self.frame_timer >= self.frame_duration {
            self.frame_timer = 0.0;
            self.frame_index = (self.frame_index + 1) % FRAME_COUNT;
        }
    }
}

pub fn render_sprites(
    d: &mut RaylibDrawHandle,
    player: &Player,
    sprites: &[AnimatedSprite],
    zbuffer: &[f32],
    screen_w: i32,
    screen_h: i32,
    creature_textures: &CreatureTextures,
) {
    let mut with_dist: Vec<(f32, &AnimatedSprite)> = sprites
        .iter()
        .filter(|s| s.alive)
        .map(|s| {
            let dx = s.x - player.x;
            let dy = s.y - player.y;
            ((dx * dx + dy * dy).sqrt(), s)
        })
        .collect();
    with_dist.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    for (dist, sprite) in with_dist {
        if dist < 0.05 {
            continue;
        }
        let dx = sprite.x - player.x;
        let dy = sprite.y - player.y;
        let angle_to_sprite = dy.atan2(dx) - player.angle;
        let angle_to_sprite = {
            let mut a = angle_to_sprite;
            while a > std::f32::consts::PI {
                a -= 2.0 * std::f32::consts::PI;
            }
            while a < -std::f32::consts::PI {
                a += 2.0 * std::f32::consts::PI;
            }
            a
        };

        if angle_to_sprite.abs() > FOV / 2.0 + 0.3 {
            continue;
        }

        let corrected_dist = dist * angle_to_sprite.cos();
        if corrected_dist < 0.1 {
            continue;
        }

        let sprite_screen_size = (screen_h as f32 / corrected_dist).min(2000.0);
        let screen_x = (screen_w as f32 / 2.0) * (1.0 + angle_to_sprite / (FOV / 2.0));

        let half = sprite_screen_size / 2.0;
        let start_x = (screen_x - half) as i32;
        let end_x = (screen_x + half) as i32;
        let start_y = (screen_h as f32 / 2.0 - half) as i32;

        let texture = &creature_textures.frames[sprite.frame_index];
        let tex_w = texture.width as f32;
        let tex_h = texture.height as f32;

        let shade = (1.0 - (corrected_dist / 12.0).min(0.6)).max(0.4);
        let v = (255.0 * shade) as u8;
        let tint = Color::new(v, v, v, 255);

        let draw_w = (end_x - start_x).max(1);
        for col in start_x.max(0)..end_x.min(screen_w) {
            if (col as usize) < zbuffer.len() && corrected_dist >= zbuffer[col as usize] {
                continue;
            }
            let local_x = col - start_x;
            let tex_x = ((local_x as f32 / draw_w as f32) * tex_w).clamp(0.0, tex_w - 1.0);

            let src = Rectangle::new(tex_x, 0.0, 1.0, tex_h);
            let dest = Rectangle::new(col as f32, start_y as f32, 1.0, sprite_screen_size);
            d.draw_texture_pro(texture, src, dest, Vector2::new(0.0, 0.0), 0.0, tint);
        }
    }
}