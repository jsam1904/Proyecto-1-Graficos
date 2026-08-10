use crate::player::Player;
use crate::raycasting::FOV;
use raylib::prelude::*;

pub struct AnimatedSprite {
    pub x: f32,
    pub y: f32,
    pub frame_colors: Vec<Color>,
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
            frame_colors: vec![
                Color::new(230, 90, 90, 255),
                Color::new(240, 130, 90, 255),
                Color::new(230, 90, 90, 255),
                Color::new(200, 60, 60, 255),
            ],
            frame_index: 0,
            frame_timer: 0.0,
            frame_duration: 0.18,
            alive: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.frame_timer += dt;
        if self.frame_timer >= self.frame_duration {
            self.frame_timer = 0.0;
            self.frame_index = (self.frame_index + 1) % self.frame_colors.len();
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
        let screen_x =
            (screen_w as f32 / 2.0) * (1.0 + angle_to_sprite / (FOV / 2.0));

        let half = sprite_screen_size / 2.0;
        let start_x = (screen_x - half) as i32;
        let end_x = (screen_x + half) as i32;
        let start_y = (screen_h as f32 / 2.0 - half) as i32;

        let color = sprite.frame_colors[sprite.frame_index];

        for col in start_x.max(0)..end_x.min(screen_w) {
            if (col as usize) < zbuffer.len() && corrected_dist >= zbuffer[col as usize] {
                continue;
            }
            d.draw_line(
                col,
                start_y.max(0),
                col,
                (start_y as f32 + sprite_screen_size) as i32,
                color,
            );
        }
    }
}