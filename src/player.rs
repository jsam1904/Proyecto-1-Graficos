use crate::map::{is_wall, MAP_HEIGHT, MAP_WIDTH};

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub move_speed: f32,
    pub rot_speed: f32,
    pub collision_radius: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            x,
            y,
            angle,
            move_speed: 3.2,
            rot_speed: 2.5,
            collision_radius: 0.20,
        }
    }

    pub fn dir(&self) -> (f32, f32) {
        (self.angle.cos(), self.angle.sin())
    }
    
    pub fn try_move(&mut self, dx: f32, dy: f32, grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT]) {
        let r = self.collision_radius;

        let new_x = self.x + dx;
        if !collides(new_x, self.y, r, grid) {
            self.x = new_x;
        }

        let new_y = self.y + dy;
        if !collides(self.x, new_y, r, grid) {
            self.y = new_y;
        }
    }

    pub fn rotate(&mut self, delta_angle: f32) {
        self.angle += delta_angle;
        let two_pi = std::f32::consts::PI * 2.0;
        self.angle = self.angle.rem_euclid(two_pi);
    }

    pub fn tile(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}

fn collides(x: f32, y: f32, r: f32, grid: &[[u8; MAP_WIDTH]; MAP_HEIGHT]) -> bool {
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

pub fn clamp_to_bounds(x: f32, y: f32) -> (f32, f32) {
    (
        x.clamp(0.5, MAP_WIDTH as f32 - 0.5),
        y.clamp(0.5, MAP_HEIGHT as f32 - 0.5),
    )
}