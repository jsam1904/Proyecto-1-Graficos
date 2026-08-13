use crate::framebuffer::Framebuffer;
use crate::map::{is_wall, MAP_HEIGHT, MAP_WIDTH};
use crate::player::Player;
use crate::raycasting::FOV;
use crate::textures::CreatureTextures;
use raylib::prelude::Color;

const FRAME_COUNT: usize = 4;

pub const RESPAWN_DELAY: f32 = 4.0;

const CHASE_RADIUS: f32 = 5.5;
const MIN_CHASE_DIST: f32 = 0.45;
const SPRITE_COLLISION_RADIUS: f32 = 0.22;

pub struct AnimatedSprite {
    pub x: f32,
    pub y: f32,
    pub frame_index: usize,
    pub frame_timer: f32,
    pub frame_duration: f32,
    pub alive: bool,
    respawn_timer: f32,
    move_speed: f32,
    wander_time: f32,
    wander_seed: f32,
}

impl AnimatedSprite {
    pub fn new(x: f32, y: f32) -> Self {
        let wander_seed = ((x * 12.9898 + y * 78.233).sin() * 43758.5453).fract();

        Self {
            x,
            y,
            frame_index: 0,
            frame_timer: 0.0,
            frame_duration: 0.28,
            alive: true,
            respawn_timer: 0.0,
            move_speed: 1.6,
            wander_time: 0.0,
            wander_seed,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.frame_timer += dt;
        if self.frame_timer >= self.frame_duration {
            self.frame_timer = 0.0;
            self.frame_index = (self.frame_index + 1) % FRAME_COUNT;
        }
    }

    pub fn update_ai(
        &mut self,
        dt: f32,
        player_x: f32,
        player_y: f32,
        grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT],
    ) {
        if !self.alive {
            return;
        }
        self.wander_time += dt;

        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        let (dir_x, dir_y) = if dist < CHASE_RADIUS && dist > MIN_CHASE_DIST {
            (dx / dist, dy / dist)
        } else if dist <= MIN_CHASE_DIST {
            (0.0, 0.0) 
        } else {
            let angle = self.wander_seed * 10.0 + self.wander_time * 0.6;
            (angle.cos(), angle.sin())
        };

        let step = self.move_speed * dt;
        try_move_sprite(&mut self.x, &mut self.y, dir_x * step, dir_y * step, grid);
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.respawn_timer = RESPAWN_DELAY;
    }

    pub fn tick_respawn(&mut self, dt: f32) -> bool {
        if self.alive {
            return false;
        }
        self.respawn_timer -= dt;
        self.respawn_timer <= 0.0
    }

    pub fn respawn_at(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.alive = true;
        self.frame_index = 0;
        self.frame_timer = 0.0;
    }
}

fn sprite_collides(x: f32, y: f32, r: f32, grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT]) -> bool {
    let checks = [
        (x - r, y - r),
        (x + r, y - r),
        (x - r, y + r),
        (x + r, y + r),
    ];
    for (cx, cy) in checks {
        if is_wall(grid, cx.floor() as i32, cy.floor() as i32) {
            return true;
        }
    }
    false
}

fn try_move_sprite(
    x: &mut f32,
    y: &mut f32,
    dx: f32,
    dy: f32,
    grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT],
) {
    let new_x = *x + dx;
    if !sprite_collides(new_x, *y, SPRITE_COLLISION_RADIUS, grid) {
        *x = new_x;
    }
    let new_y = *y + dy;
    if !sprite_collides(*x, new_y, SPRITE_COLLISION_RADIUS, grid) {
        *y = new_y;
    }
}

pub fn render_sprites_fb(
    fb: &mut Framebuffer,
    player: &Player,
    sprites: &[AnimatedSprite],
    zbuffer: &[f32],
    creature_textures: &CreatureTextures,
) {
    let screen_w = fb.width;
    let screen_h = fb.height;

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
        let end_y = start_y + sprite_screen_size as i32;

        let texture = &creature_textures.frames[sprite.frame_index];
        let tex_w = texture.width;
        let tex_h = texture.height;

        let shade = (1.0 - (corrected_dist / 12.0).min(0.6)).max(0.4);

        let draw_w = (end_x - start_x).max(1);
        let draw_h = (end_y - start_y).max(1);

        for col in start_x.max(0)..end_x.min(screen_w) {
            if (col as usize) < zbuffer.len() && corrected_dist >= zbuffer[col as usize] {
                continue;
            }
            let local_x = col - start_x;
            let tex_x = ((local_x as f32 / draw_w as f32) * tex_w as f32) as i32;

            for row in start_y.max(0)..end_y.min(screen_h) {
                let local_y = row - start_y;
                let tex_y = ((local_y as f32 / draw_h as f32) * tex_h as f32) as i32;
                let c = texture.get(tex_x, tex_y);
                if c.a == 0 {
                    continue;
                }
                let shaded = Color::new(
                    (c.r as f32 * shade) as u8,
                    (c.g as f32 * shade) as u8,
                    (c.b as f32 * shade) as u8,
                    255,
                );
                fb.set_pixel(col, row, shaded);
            }
        }
    }
}