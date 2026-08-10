use crate::map::{wall_type_at, MAP_HEIGHT, MAP_WIDTH};
use crate::player::Player;
use raylib::prelude::*;

pub const FOV: f32 = std::f32::consts::PI / 3.0;

pub struct RayHit {
    pub distance: f32,
    pub wall_type: u8,
    pub side: u8, 
}

pub fn wall_color(wall_type: u8, side: u8) -> Color {
    let base = match wall_type {
        1 => Color::new(180, 60, 60, 255),
        2 => Color::new(60, 130, 180, 255),
        3 => Color::new(90, 170, 90, 255),
        4 => Color::new(200, 170, 60, 255),
        _ => Color::new(150, 150, 150, 255), 
    };
    if side == 1 {
        Color::new(
            (base.r as f32 * 0.7) as u8,
            (base.g as f32 * 0.7) as u8,
            (base.b as f32 * 0.7) as u8,
            255,
        )
    } else {
        base
    }
}

pub fn cast_ray(px: f32, py: f32, ray_angle: f32, grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT],) -> RayHit {
    let ray_dir_x = ray_angle.cos();
    let ray_dir_y = ray_angle.sin();

    let mut map_x = px as i32;
    let mut map_y = py as i32;

    let delta_dist_x = if ray_dir_x == 0.0 { 1e30 } else { (1.0 / ray_dir_x).abs() };
    let delta_dist_y = if ray_dir_y == 0.0 { 1e30 } else { (1.0 / ray_dir_y).abs() };

    let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
        (-1, (px - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - px) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
        (-1, (py - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - py) * delta_dist_y)
    };

    let mut side: u8;
    let mut wall_type;
    let mut hit = false;
    let mut safety = 0;

    loop {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }

        wall_type = wall_type_at(grid, map_x, map_y);
        if wall_type != 0 {
            hit = true;
        }

        safety += 1;
        if hit || safety > (MAP_WIDTH + MAP_HEIGHT) * 2 {
            let perp_dist = if side == 0 {
                (map_x as f32 - px + (1 - step_x) as f32 / 2.0) / ray_dir_x
            } else {
                (map_y as f32 - py + (1 - step_y) as f32 / 2.0) / ray_dir_y
            };
            return RayHit {
                distance: perp_dist.max(0.0001),
                wall_type,
                side,
            };
        }
    }
}

pub fn render_scene(d: &mut RaylibDrawHandle, player: &Player, grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT], screen_w: i32, screen_h: i32, zbuffer: &mut Vec<f32>,) {
    d.draw_rectangle(0, 0, screen_w, screen_h / 2, Color::new(40, 40, 50, 255));
    d.draw_rectangle(
        0,
        screen_h / 2,
        screen_w,
        screen_h / 2,
        Color::new(60, 60, 60, 255),
    );

    for col in 0..screen_w {
        let camera_x = 2.0 * col as f32 / screen_w as f32 - 1.0;
        let ray_angle = player.angle + camera_x * (FOV / 2.0);

        let hit = cast_ray(player.x, player.y, ray_angle, grid);

        let corrected_dist = hit.distance * (ray_angle - player.angle).cos();
        let corrected_dist = corrected_dist.max(0.0001);

        let line_height = (screen_h as f32 / corrected_dist) as i32;
        let draw_start = (-line_height / 2 + screen_h / 2).max(0);
        let draw_end = (line_height / 2 + screen_h / 2).min(screen_h - 1);

        let color = wall_color(hit.wall_type, hit.side);
        let shade = (1.0 - (corrected_dist / 12.0).min(0.85)).max(0.15);
        let shaded = Color::new(
            (color.r as f32 * shade) as u8,
            (color.g as f32 * shade) as u8,
            (color.b as f32 * shade) as u8,
            255,
        );

        d.draw_line(col, draw_start, col, draw_end, shaded);

        if (col as usize) < zbuffer.len() {
            zbuffer[col as usize] = corrected_dist;
        }
    }
}

pub fn render_minimap(d: &mut RaylibDrawHandle, player: &Player, grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT], screen_w: i32,) {
    let tile_px = 6;
    let margin = 12;
    let map_w = MAP_WIDTH as i32 * tile_px;
    let map_h = MAP_HEIGHT as i32 * tile_px;
    let origin_x = screen_w - map_w - margin;
    let origin_y = margin;

    d.draw_rectangle(
        origin_x - 4,
        origin_y - 4,
        map_w + 8,
        map_h + 8,
        Color::new(0, 0, 0, 160),
    );

    for row in 0..MAP_HEIGHT {
        for col in 0..MAP_WIDTH {
            let t = grid[row][col];
            let color = if t == 0 {
                Color::new(30, 30, 30, 255)
            } else {
                wall_color(t, 0)
            };
            d.draw_rectangle(
                origin_x + col as i32 * tile_px,
                origin_y + row as i32 * tile_px,
                tile_px - 1,
                tile_px - 1,
                color,
            );
        }
    }

    let px = origin_x + (player.x * tile_px as f32) as i32;
    let py = origin_y + (player.y * tile_px as f32) as i32;
    d.draw_circle(px, py, 3.0, Color::YELLOW);
    let (dx, dy) = player.dir();
    d.draw_line(
        px,
        py,
        px + (dx * 10.0) as i32,
        py + (dy * 10.0) as i32,
        Color::YELLOW,
    );
}