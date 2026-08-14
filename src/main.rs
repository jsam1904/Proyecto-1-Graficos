mod audio;
mod framebuffer;
mod map;
mod player;
mod raycasting;
mod recorder;
mod sprite;
mod state;
mod textures;
mod weapon;

use audio::AudioManager;
use framebuffer::Framebuffer;
use map::get_levels;
use player::Player;
use raylib::audio::RaylibAudio;
use raylib::prelude::*;
use recorder::GifRecorder;
use sprite::{render_sprites_fb, AnimatedSprite};
use state::GameState;
use textures::{CreatureTextures, WallTextures};
use weapon::Weapon;

const SCREEN_W: i32 = 960;
const SCREEN_H: i32 = 540;
const NUM_ACTIVE_ENEMIES: usize = 3;
const GAME_OVER_DISTANCE: f32 = 0.45;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_W, SCREEN_H)
        .title("Rust Raycaster - UVG")
        .vsync()
        .build();

    rl.set_target_fps(60);
    rl.set_exit_key(None);

    let levels = get_levels();
    let mut current_level: usize = 0;
    let mut selected_menu_option: usize = 0;

    let mut state = GameState::Welcome;

    let audio = RaylibAudio::init_audio_device().expect("No se pudo inicializar el dispositivo de audio");
    let mut audio_mgr = AudioManager::new(&audio);

    let mut player = Player::new(
        levels[current_level].player_start.0,
        levels[current_level].player_start.1,
        levels[current_level].player_start_angle,
    );

    let mut sprites: Vec<AnimatedSprite> =
        spawn_initial_sprites(&levels[current_level].spawn_points, NUM_ACTIVE_ENEMIES);
    let mut spawn_cursor: usize = NUM_ACTIVE_ENEMIES;
    let mut zbuffer: Vec<f32> = vec![1e30; SCREEN_W as usize];

    let wall_textures = WallTextures::load();
    let creature_textures = CreatureTextures::load();
    let mut weapon = Weapon::new();

    let mut framebuffer = Framebuffer::new(&mut rl, &thread, SCREEN_W, SCREEN_H);

    let mut gif_recorder: Option<GifRecorder> = None;
    let mut has_recorded_once = false;
    let mut minimap_fullscreen = false;
    let mut cursor_locked = false;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        audio_mgr.update();

        match state {
            GameState::Welcome => {
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                    || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
                {
                    state = GameState::LevelSelect;
                }
            }
            GameState::LevelSelect => {
                if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                    selected_menu_option = (selected_menu_option + 1) % levels.len();
                }
                if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                    selected_menu_option =
                        (selected_menu_option + levels.len() - 1) % levels.len();
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    current_level = selected_menu_option;
                    player = Player::new(
                        levels[current_level].player_start.0,
                        levels[current_level].player_start.1,
                        levels[current_level].player_start_angle,
                    );
                    sprites =
                        spawn_initial_sprites(&levels[current_level].spawn_points, NUM_ACTIVE_ENEMIES);
                    spawn_cursor = NUM_ACTIVE_ENEMIES;
                    state = GameState::Playing;

                    if !has_recorded_once {
                        has_recorded_once = true;
                        gif_recorder = GifRecorder::start("demo.gif", SCREEN_W, SCREEN_H, 12.0, 8.0);
                        audio_mgr.start_music();
                    }
                }
            }
            GameState::Playing => {
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    state = GameState::LevelSelect;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_M) {
                    minimap_fullscreen = !minimap_fullscreen;
                }

                update_playing(&mut rl, &mut player, &levels[current_level].grid, dt);

                let is_moving = rl.is_key_down(KeyboardKey::KEY_W)
                    || rl.is_key_down(KeyboardKey::KEY_S)
                    || rl.is_key_down(KeyboardKey::KEY_A)
                    || rl.is_key_down(KeyboardKey::KEY_D);
                weapon.update(dt, is_moving);

                for s in sprites.iter_mut() {
                    if s.alive {
                        s.update(dt);
                        s.update_ai(dt, player.x, player.y, &levels[current_level].grid);
                    } else if s.tick_respawn(dt) {
                        let points = &levels[current_level].spawn_points;
                        if !points.is_empty() {
                            let (sx, sy) = points[spawn_cursor % points.len()];
                            spawn_cursor += 1;
                            s.respawn_at(sx, sy);
                        }
                    }
                }

                let caught_by_enemy = sprites.iter().any(|s| {
                    if !s.alive {
                        return false;
                    }
                    let dx = s.x - player.x;
                    let dy = s.y - player.y;
                    (dx * dx + dy * dy).sqrt() < GAME_OVER_DISTANCE
                });
                if caught_by_enemy {
                    state = GameState::GameOver;
                }

                if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                    || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
                {
                    try_shoot(&player, &mut sprites);
                    weapon.trigger_shot();
                    audio_mgr.play_shot();
                }

                let (tx, ty) = player.tile();
                if (tx, ty) == levels[current_level].goal_tile {
                    state = GameState::Success;
                }
            }
            GameState::Success => {
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state = GameState::LevelSelect;
                }
            }
            GameState::GameOver => {
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    player = Player::new(
                        levels[current_level].player_start.0,
                        levels[current_level].player_start.1,
                        levels[current_level].player_start_angle,
                    );
                    sprites =
                        spawn_initial_sprites(&levels[current_level].spawn_points, NUM_ACTIVE_ENEMIES);
                    spawn_cursor = NUM_ACTIVE_ENEMIES;
                    state = GameState::Playing;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    state = GameState::LevelSelect;
                }
            }
        }

        let should_lock_cursor = state == GameState::Playing;

        if should_lock_cursor && !cursor_locked {
            rl.hide_cursor();
            rl.set_mouse_position(Vector2::new(SCREEN_W as f32 / 2.0, SCREEN_H as f32 / 2.0));
            cursor_locked = true;
        } else if !should_lock_cursor && cursor_locked {
            rl.show_cursor();
            cursor_locked = false;
        }

        if state == GameState::Playing {
            raycasting::render_scene_fb(
                &mut framebuffer,
                &player,
                &levels[current_level].grid,
                &mut zbuffer,
                &wall_textures,
            );
            render_sprites_fb(&mut framebuffer, &player, &sprites, &zbuffer, &creature_textures);
            raycasting::render_minimap_fb(
                &mut framebuffer,
                &player,
                &levels[current_level].grid,
                minimap_fullscreen,
            );
            if !minimap_fullscreen {
                weapon.draw_fb(&mut framebuffer);
                weapon::draw_crosshair_fb(&mut framebuffer);
            }
            framebuffer.upload();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        match state {
            GameState::Welcome => draw_welcome(&mut d),
            GameState::LevelSelect => draw_level_select(&mut d, &levels, selected_menu_option),
            GameState::Playing => {
                framebuffer.present(&mut d);
                d.draw_fps(10, 10);
                d.draw_text(
                    "ESC: menu | WASD: mover | Mouse: rotar | Click/Espacio: disparar | M: mapa completo",
                    10,
                    SCREEN_H - 24,
                    16,
                    Color::WHITE,
                );
            }
            GameState::Success => draw_success(&mut d, &levels[current_level].name),
            GameState::GameOver => draw_game_over(&mut d, &levels[current_level].name),
        }
        drop(d);

        if let Some(recorder) = gif_recorder.as_mut() {
            recorder.update(dt, &mut rl, &thread);
            if !recorder.is_active() {
                gif_recorder = None;
            }
        }
    }
}

fn update_playing(
    rl: &mut RaylibHandle,
    player: &mut Player,
    grid: &[[u8; map::MAP_WIDTH]; map::MAP_HEIGHT],
    dt: f32,
) {

    const MOUSE_SENSITIVITY: f32 = 0.0001;
    let center_x = SCREEN_W as f32 / 2.0;
    let center_y = SCREEN_H as f32 / 2.0;
    let mouse_pos = rl.get_mouse_position();
    let delta_x = mouse_pos.x - center_x;
    if delta_x.abs() > 0.5 {
        player.rotate(delta_x * MOUSE_SENSITIVITY);
    }
    rl.set_mouse_position(Vector2::new(center_x, center_y));

    let (dir_x, dir_y) = player.dir();
    let (strafe_x, strafe_y) = (-dir_y, dir_x);

    let mut move_x = 0.0;
    let mut move_y = 0.0;
    let speed = player.move_speed * dt;

    if rl.is_key_down(KeyboardKey::KEY_W) {
        move_x += dir_x * speed;
        move_y += dir_y * speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        move_x -= dir_x * speed;
        move_y -= dir_y * speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) {
        move_x -= strafe_x * speed;
        move_y -= strafe_y * speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        move_x += strafe_x * speed;
        move_y += strafe_y * speed;
    }

    player.try_move(move_x, move_y, grid);
}

fn try_shoot(player: &Player, sprites: &mut Vec<AnimatedSprite>) {
    const SHOOT_CONE: f32 = 0.15;
    const MAX_RANGE: f32 = 10.0;

    let mut best: Option<(usize, f32)> = None;
    for (i, s) in sprites.iter().enumerate() {
        if !s.alive {
            continue;
        }
        let dx = s.x - player.x;
        let dy = s.y - player.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > MAX_RANGE {
            continue;
        }
        let angle_to = dy.atan2(dx);
        let mut diff = angle_to - player.angle;
        while diff > std::f32::consts::PI {
            diff -= 2.0 * std::f32::consts::PI;
        }
        while diff < -std::f32::consts::PI {
            diff += 2.0 * std::f32::consts::PI;
        }
        if diff.abs() <= SHOOT_CONE {
            if best.map_or(true, |(_, d)| dist < d) {
                best = Some((i, dist));
            }
        }
    }
    if let Some((i, _)) = best {
        sprites[i].kill();
    }
}

fn spawn_initial_sprites(spawn_points: &[(f32, f32)], count: usize) -> Vec<AnimatedSprite> {
    if spawn_points.is_empty() {
        return Vec::new();
    }
    (0..count)
        .map(|i| {
            let (x, y) = spawn_points[i % spawn_points.len()];
            AnimatedSprite::new(x, y)
        })
        .collect()
}

fn draw_welcome(d: &mut RaylibDrawHandle) {
    d.clear_background(Color::new(20, 20, 30, 255));
    let title = "RUST RAYCASTER";
    d.draw_text(title, SCREEN_W / 2 - 180, 180, 48, Color::YELLOW);
    d.draw_text(
        "Proyecto de Graficas por Computadora - UVG",
        SCREEN_W / 2 - 220,
        250,
        20,
        Color::LIGHTGRAY,
    );
    d.draw_text(
        "Presiona ENTER o ESPACIO para continuar",
        SCREEN_W / 2 - 200,
        320,
        20,
        Color::WHITE,
    );
}

fn draw_level_select(d: &mut RaylibDrawHandle, levels: &[map::Level], selected: usize) {
    d.clear_background(Color::new(20, 20, 30, 255));
    d.draw_text("Selecciona un nivel", SCREEN_W / 2 - 150, 100, 32, Color::WHITE);
    for (i, lvl) in levels.iter().enumerate() {
        let color = if i == selected { Color::YELLOW } else { Color::GRAY };
        let prefix = if i == selected { "> " } else { "  " };
        d.draw_text(
            &format!("{}{}", prefix, lvl.name),
            SCREEN_W / 2 - 150,
            180 + i as i32 * 40,
            24,
            color,
        );
    }
    d.draw_text(
        "Flechas: navegar | ENTER: jugar",
        SCREEN_W / 2 - 150,
        SCREEN_H - 60,
        18,
        Color::LIGHTGRAY,
    );
}

fn draw_success(d: &mut RaylibDrawHandle, level_name: &str) {
    d.clear_background(Color::new(15, 40, 20, 255));
    d.draw_text("NIVEL COMPLETADO", SCREEN_W / 2 - 220, 200, 42, Color::GREEN);
    d.draw_text(level_name, SCREEN_W / 2 - 100, 260, 20, Color::WHITE);
    d.draw_text(
        "Presiona ENTER para volver al menu",
        SCREEN_W / 2 - 200,
        330,
        20,
        Color::LIGHTGRAY,
    );
}

fn draw_game_over(d: &mut RaylibDrawHandle, level_name: &str) {
    d.clear_background(Color::new(40, 12, 12, 255));
    d.draw_text("GAME OVER", SCREEN_W / 2 - 160, 190, 48, Color::RED);
    d.draw_text(
        &format!("Te atrapo un enemigo en: {}", level_name),
        SCREEN_W / 2 - 210,
        260,
        20,
        Color::WHITE,
    );
    d.draw_text(
        "ENTER: reintentar nivel  |  ESC: volver al menu",
        SCREEN_W / 2 - 220,
        330,
        20,
        Color::LIGHTGRAY,
    );
}