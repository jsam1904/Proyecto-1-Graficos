pub const MAP_WIDTH: usize = 16;
pub const MAP_HEIGHT: usize = 16;

#[rustfmt::skip]
pub const LEVEL_1: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,2,2,2,0,0,0,1],
    [1,0,1,1,1,0,0,0,0,2,0,2,0,0,0,1],
    [1,0,1,0,0,0,0,0,0,2,0,2,0,0,0,1],
    [1,0,1,0,3,3,3,0,0,2,0,2,0,0,0,1],
    [1,0,0,0,3,0,3,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,3,0,3,0,0,0,0,0,0,4,0,1],
    [1,0,0,0,3,3,3,0,0,0,0,0,0,4,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,4,0,1],
    [1,0,0,0,0,0,1,1,1,0,0,0,0,4,0,1],
    [1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,1,0,2,2,0,3,3,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
];

// Nivel 2 de ejemplo (más abierto, otro layout). Duplícalo/edítalo a gusto.
#[rustfmt::skip]
pub const LEVEL_2: [[u8; MAP_WIDTH]; MAP_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,2,2,2,2,0,0,0,0,3,3,3,3,0,1],
    [1,0,2,0,0,2,0,0,0,0,3,0,0,3,0,1],
    [1,0,2,0,0,2,0,0,0,0,3,0,0,3,0,1],
    [1,0,2,2,0,2,0,0,0,0,3,2,0,3,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,4,4,4,4,4,4,0,0,0,0,0,1],
    [1,0,0,0,4,0,0,0,0,4,0,0,0,0,0,1],
    [1,0,0,0,4,0,0,0,0,4,0,0,0,0,0,1],
    [1,0,0,0,4,4,0,0,4,4,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,3,0,0,0,0,0,0,0,0,0,2,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
];

pub struct Level {
    pub grid: [[u8; MAP_WIDTH]; MAP_HEIGHT],
    pub name: &'static str,
    pub player_start: (f32, f32),
    pub player_start_angle: f32,
    // Casilla (col, row) que, al pisarla, dispara la pantalla de éxito.
    pub goal_tile: (usize, usize),
}

pub fn get_levels() -> Vec<Level> {
    vec![
        Level {
            grid: LEVEL_1,
            name: "Nivel 1: Los Pasillos",
            player_start: (2.5, 1.5),
            player_start_angle: 0.0,
            goal_tile: (13, 7),
        },
        Level {
            grid: LEVEL_2,
            name: "Nivel 2: La Arena",
            player_start: (2.5, 1.5),
            player_start_angle: 0.0,
            goal_tile: (6, 8),
        },
    ]
}

#[inline]
pub fn is_wall(grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT], x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x as usize >= MAP_WIDTH || y as usize >= MAP_HEIGHT {
        return true; // fuera del mapa cuenta como pared (evita index out of range)
    }
    grid[y as usize][x as usize] != 0
}

#[inline]
pub fn wall_type_at(grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT], x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x as usize >= MAP_WIDTH || y as usize >= MAP_HEIGHT {
        return 1;
    }
    grid[y as usize][x as usize]
}