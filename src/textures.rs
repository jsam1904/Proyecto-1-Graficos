use raylib::prelude::*;

pub const TEX_SIZE: i32 = 64;

fn darker(c: Color, factor: f32) -> Color {
    Color::new(
        (c.r as f32 * factor) as u8,
        (c.g as f32 * factor) as u8,
        (c.b as f32 * factor) as u8,
        255,
    )
}

fn brick_pattern(base: Color, mortar: Color) -> Image {
    let mut img = Image::gen_image_color(TEX_SIZE, TEX_SIZE, mortar);
    let brick_w = 16;
    let brick_h = 8;
    let mortar_th = 2;

    let mut y = 0;
    let mut row = 0;
    while y < TEX_SIZE {
        let offset = if row % 2 == 0 { 0 } else { brick_w / 2 };
        let mut x = -offset;
        while x < TEX_SIZE {
            let bx = x + mortar_th;
            let by = y + mortar_th;
            let bw = brick_w - mortar_th * 2;
            let bh = brick_h - mortar_th * 2;
            if bw > 0 && bh > 0 {
                let shade = 0.85 + ((x.abs() * 7 + y * 3) % 5) as f32 * 0.03;
                img.draw_rectangle(bx, by, bw, bh, darker(base, shade));
            }
            x += brick_w;
        }
        y += brick_h;
        row += 1;
    }
    img
}

fn panel_pattern(base: Color, line: Color) -> Image {
    let mut img = Image::gen_image_color(TEX_SIZE, TEX_SIZE, base);
    let panel = 32;
    let mut y = 0;
    while y < TEX_SIZE {
        img.draw_rectangle(0, y, TEX_SIZE, 2, line);
        y += panel;
    }
    let mut x = 0;
    while x < TEX_SIZE {
        img.draw_rectangle(x, 0, 2, TEX_SIZE, line);
        x += panel;
    }
    // remaches
    let rivet = darker(base, 1.3);
    for py in (4..TEX_SIZE).step_by(panel as usize) {
        for px in (4..TEX_SIZE).step_by(panel as usize) {
            img.draw_circle(px, py, 2, rivet);
            img.draw_circle(px + panel - 8, py, 2, rivet);
        }
    }
    img
}

fn stone_pattern(base: Color) -> Image {
    let mut img = Image::gen_image_color(TEX_SIZE, TEX_SIZE, darker(base, 0.6));
    let block = 12;
    let mut y = 0;
    let mut row = 0;
    while y < TEX_SIZE {
        let offset = if row % 2 == 0 { 0 } else { block / 2 };
        let mut x = -offset;
        while x < TEX_SIZE {
            let n = ((x * 13 + y * 7).rem_euclid(11)) as f32 / 11.0; // pseudo-ruido determinista
            let shade = 0.75 + n * 0.4;
            img.draw_rectangle(x + 1, y + 1, block - 2, block - 2, darker(base, shade));
            x += block;
        }
        y += block;
        row += 1;
    }
    img
}

pub struct WallTextures {
    pub tex_1: Texture2D,
    pub tex_2: Texture2D,
    pub tex_3: Texture2D,
    pub tex_4: Texture2D,
    pub tex_default: Texture2D,
}

impl WallTextures {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let img1 = brick_pattern(Color::new(178, 74, 58, 255), Color::new(90, 60, 55, 255));
        let img2 = panel_pattern(Color::new(60, 110, 150, 255), Color::new(30, 60, 90, 255));
        let img3 = stone_pattern(Color::new(95, 140, 95, 255));
        let img4 = panel_pattern(Color::new(190, 155, 60, 255), Color::new(120, 95, 30, 255));
        let img_default = brick_pattern(Color::new(140, 140, 140, 255), Color::new(80, 80, 80, 255));

        Self {
            tex_1: rl.load_texture_from_image(thread, &img1).expect("tex1"),
            tex_2: rl.load_texture_from_image(thread, &img2).expect("tex2"),
            tex_3: rl.load_texture_from_image(thread, &img3).expect("tex3"),
            tex_4: rl.load_texture_from_image(thread, &img4).expect("tex4"),
            tex_default: rl.load_texture_from_image(thread, &img_default).expect("texd"),
        }
    }

    pub fn get(&self, wall_type: u8) -> &Texture2D {
        match wall_type {
            1 => &self.tex_1,
            2 => &self.tex_2,
            3 => &self.tex_3,
            4 => &self.tex_4,
            _ => &self.tex_default,
        }
    }
}

pub fn generate_pistol_image() -> Image {
    let w = 100;
    let h = 130;
    let mut img = Image::gen_image_color(w, h, Color::new(0, 0, 0, 0));

    let metal = Color::new(75, 75, 80, 255);
    let metal_dark = Color::new(42, 42, 47, 255);
    let metal_light = Color::new(120, 120, 128, 255);
    let grip = Color::new(55, 38, 24, 255);
    let grip_dark = Color::new(38, 25, 15, 255);
    let grip_light = Color::new(70, 48, 30, 255);
    let sight = Color::new(20, 20, 20, 255);
    let transparent = Color::new(0, 0, 0, 0);

    let cx = w / 2; 

    img.draw_rectangle(cx - 27, 8, 54, 26, metal);
    img.draw_rectangle(cx - 27, 8, 54, 5, metal_light);
    img.draw_rectangle(cx - 24, 30, 48, 4, metal_dark);

    img.draw_rectangle(cx - 8, 2, 16, 8, metal_dark);
    img.draw_rectangle(cx - 5, 4, 4, 4, sight);
    img.draw_rectangle(cx + 1, 4, 4, 4, sight);

    img.draw_rectangle(cx - 22, 34, 44, 16, metal_dark);
    img.draw_rectangle(cx - 22, 34, 44, 3, metal);

    img.draw_rectangle(cx - 12, 50, 24, 8, metal_dark);
    img.draw_rectangle(cx - 9, 50, 18, 12, transparent);
    img.draw_rectangle(cx - 12, 58, 5, 6, metal_dark);
    img.draw_rectangle(cx + 7, 58, 5, 6, metal_dark);

    img.draw_rectangle(cx - 16, 58, 32, 12, grip);
    img.draw_rectangle(cx - 18, 70, 36, 14, grip);
    img.draw_rectangle(cx - 20, 84, 40, 20, grip);
    img.draw_rectangle(cx - 10, 70, 20, 32, grip_dark);
    img.draw_rectangle(cx - 20, 84, 5, 20, grip_light);
    img.draw_rectangle(cx + 15, 84, 5, 20, grip_light);

    img.draw_rectangle(cx - 22, 104, 44, 6, grip_dark);

    img
}

fn generate_creature_frame(blink: bool, pupil_shift: i32) -> Image {
    let size = 48;
    let mut img = Image::gen_image_color(size, size, Color::new(0, 0, 0, 0));
    let cx = 24;
    let cy = 27;

    let body_dark = Color::new(35, 100, 55, 255);
    let body = Color::new(85, 185, 95, 255);
    let body_light = Color::new(140, 225, 145, 255);

    img.draw_circle(cx, cy + 2, 19, body_dark);
    img.draw_circle(cx, cy, 17, body);
    img.draw_circle(cx - 6, cy - 7, 6, body_light);

    let eye_y = cy - 4;
    for &ex in &[cx - 7, cx + 7] {
        if blink {
            img.draw_rectangle(ex - 4, eye_y - 1, 8, 2, Color::new(20, 45, 25, 255));
        } else {
            img.draw_circle(ex, eye_y, 5, Color::new(250, 250, 240, 255));
            img.draw_circle(ex + pupil_shift, eye_y, 2, Color::new(10, 10, 15, 255));
        }
    }

    img
}

pub struct CreatureTextures {
    pub frames: Vec<Texture2D>,
}

impl CreatureTextures {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let images = vec![
            generate_creature_frame(false, -2),
            generate_creature_frame(false, 0),
            generate_creature_frame(false, 2),
            generate_creature_frame(true, 0),
        ];
        let frames = images
            .iter()
            .map(|img| rl.load_texture_from_image(thread, img).expect("creature tex"))
            .collect();
        Self { frames }
    }
}