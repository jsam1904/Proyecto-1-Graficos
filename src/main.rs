mod map;
mod player;
mod raycasting;
mod sprite;
mod state;
mod textures;
mod weapon;

use map::get_levels;
use player::Player;
use raylib::prelude::*;
use sprite::{render_sprites, AnimatedSprite};
use state::GameState;
use textures::{CreatureTextures, WallTextures};
use weapon::Weapon;

const SCREEN_W: i32 = 960;
const SCREEN_H: i32 = 540;

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

    let mut player = Player::new(
        levels[current_level].player_start.0,
        levels[current_level].player_start.1,
        levels[current_level].player_start_angle,
    );

    let mut sprites: Vec<AnimatedSprite> = spawn_sprites_for_level(current_level);
    let mut zbuffer: Vec<f32> = vec![1e30; SCREEN_W as usize];

    let wall_textures = WallTextures::load(&mut rl, &thread);
    let creature_textures = CreatureTextures::load(&mut rl, &thread);
    let mut weapon = Weapon::new(&mut rl, &thread);

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

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
                    sprites = spawn_sprites_for_level(current_level);
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    state = GameState::LevelSelect;
                }

                update_playing(&mut rl, &mut player, &levels[current_level].grid, dt);

                let is_moving = rl.is_key_down(KeyboardKey::KEY_W)
                    || rl.is_key_down(KeyboardKey::KEY_S)
                    || rl.is_key_down(KeyboardKey::KEY_A)
                    || rl.is_key_down(KeyboardKey::KEY_D);
                weapon.update(dt, is_moving);

                for s in sprites.iter_mut() {
                    s.update(dt);
                }

                // Disparo: click izquierdo o SPACE. Revisa el sprite más cercano
                // dentro de un cono angular pequeño frente al jugador.
                if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                    || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
                {
                    try_shoot(&player, &mut sprites);
                    weapon.trigger_shot();
                    // TODO efecto de sonido de disparo aquí (sound.play())
                }

                // Condición de victoria: pisar la casilla meta del nivel
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
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        match state {
            GameState::Welcome => draw_welcome(&mut d),
            GameState::LevelSelect => draw_level_select(&mut d, &levels, selected_menu_option),
            GameState::Playing => {
                raycasting::render_scene(
                    &mut d,
                    &player,
                    &levels[current_level].grid,
                    SCREEN_W,
                    SCREEN_H,
                    &mut zbuffer,
                    &wall_textures,
                );
                render_sprites(&mut d, &player, &sprites, &zbuffer, SCREEN_W, SCREEN_H, &creature_textures);
                raycasting::render_minimap(&mut d, &player, &levels[current_level].grid, SCREEN_W);
                weapon.draw(&mut d, SCREEN_W, SCREEN_H);
                weapon::draw_crosshair(&mut d, SCREEN_W, SCREEN_H);
                d.draw_fps(10, 10);
                d.draw_text("ESC: menu | WASD: mover | Flechas: rotar | Click/Espacio: disparar", 10, SCREEN_H - 24, 16, Color::WHITE);
            }
            GameState::Success => draw_success(&mut d, &levels[current_level].name),
        }
    }
}

fn update_playing(
    rl: &mut RaylibHandle,
    player: &mut Player,
    grid: &[[u8; map::MAP_WIDTH]; map::MAP_HEIGHT],
    dt: f32,
) {
    // --- Rotación con flechas izquierda/derecha ---
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.rotate(-player.rot_speed * dt);
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.rotate(player.rot_speed * dt);
    }

    // --- Movimiento WASD con colisión (no atraviesa paredes) ---
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
        sprites[i].alive = false;
    }
}

fn spawn_sprites_for_level(level_index: usize) -> Vec<AnimatedSprite> {
    match level_index {
        0 => vec![
            AnimatedSprite::new(6.5, 3.5),
            AnimatedSprite::new(10.5, 10.5),
        ],
        _ => vec![
            AnimatedSprite::new(3.5, 3.5),
            AnimatedSprite::new(12.5, 12.5),
        ],
    }
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