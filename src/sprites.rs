use std::f32::consts::PI;
fn normalize_angle(mut a: f32) -> f32 {
    while a >  PI { a -= 2.0 * PI; }
    while a < -PI { a += 2.0 * PI; }
    a
}
use raylib::prelude::*;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::path::Path;
use image::io::Reader as ImageReader;
use image::GenericImageView;

#[derive(Clone, Debug)]
struct CpuImage {
    w: u32,
    h: u32,
    data: Vec<u8>, // RGBA8, row-major
}

fn load_any_image_rgba8(path: &str) -> Result<CpuImage, String> {
    let reader = ImageReader::open(path)
        .map_err(|e| format!("open {}: {}", path, e))?
        .with_guessed_format()
        .map_err(|e| format!("format {}: {}", path, e))?;
    let dynimg = reader.decode().map_err(|e| format!("decode {}: {}", path, e))?;
    let rgba = dynimg.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(CpuImage { w, h, data: rgba.into_raw() })
}

use crate::maze::{Maze, is_walkable};
use crate::player::Player;
use crate::framebuffer::Framebuffer;

#[derive(Clone, Debug)]
pub struct Sprite {
    pub x: f32,    // mundo px
    pub y: f32,    // mundo px
    pub size: f32, // tamaño “alto” en px en el mundo (luego se proyecta)
    pub ch: char,  // 'c' = coin, 'P' = polis, 'Q' = quiz
    pub alive: bool,
}

pub struct SpriteManager {
    images: HashMap<char, CpuImage>, // muestreo CPU RGBA8
}

impl SpriteManager {
    pub fn new() -> Result<Self, String> {
        let mut images = HashMap::new();

        // Carga tus sprites con soporte WEBP/PNG usando el crate `image`
        // coin: prefer WEBP, fallback a PNG
        let coin = if Path::new("assets/coin.webp").exists() {
            load_any_image_rgba8("assets/coin.webp")?
        } else {
            load_any_image_rgba8("assets/coin.png")?
        };

        // polis: PNG (try police.png then police.png)
        let polis = if Path::new("assets/police.png").exists() {
            load_any_image_rgba8("assets/police.png")?
        } else {
            load_any_image_rgba8("assets/police.png")?
        };

        // portal abierto: prefer keys.webp (tu archivo), luego quiz.webp, luego quiz.png
        let quiz = if Path::new("assets/keys.webp").exists() {
            load_any_image_rgba8("assets/keys.webp")?
        } else if Path::new("assets/quiz.webp").exists() {
            load_any_image_rgba8("assets/quiz.webp")?
        } else {
            load_any_image_rgba8("assets/quiz.png")?
        };

        images.insert('c', coin);
        images.insert('P', polis);
        images.insert('Q', quiz);

        Ok(Self { images })
    }

    pub fn get_size(&self, ch: char) -> (u32, u32) {
        if let Some(img) = self.images.get(&ch) {
            (img.w, img.h)
        } else {
            (1, 1)
        }
    }

    pub fn sample(&self, ch: char, tx: u32, ty: u32) -> Color {
        if let Some(img) = self.images.get(&ch) {
            let w = img.w.max(1);
            let h = img.h.max(1);
            let x = tx.min(w - 1);
            let y = ty.min(h - 1);
            let idx = ((y * w + x) as usize) * 4;
            if idx + 3 < img.data.len() {
                let r = img.data[idx];
                let g = img.data[idx + 1];
                let b = img.data[idx + 2];
                let a = img.data[idx + 3];
                return Color::new(r, g, b, a);
            }
        }
        Color::WHITE
    }
}

/// Selecciona `n` celdas libres al azar y crea sprites “moneda”
pub fn spawn_coins(maze: &Maze, block_size: usize, n: usize) -> Vec<Sprite> {
    let mut free_cells = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if is_walkable(c) {
                free_cells.push((i, j));
            }
        }
    }
    free_cells.shuffle(&mut thread_rng());

    let mut out = Vec::new();
    for (i, j) in free_cells.into_iter().take(n) {
        let x = (i * block_size + block_size / 2) as f32;
        let y = (j * block_size + block_size / 2) as f32;
        out.push(Sprite {
            x,
            y,
            size: (block_size as f32) * 0.4, // moneda más chica (40% de la celda)
            ch: 'c',
            alive: true,
        });
    }
    out
}

/// Render de sprites con proyección + z-buffer (occlusión por paredes)
pub fn render_sprites(
    framebuffer: &mut Framebuffer,
    player: &Player,
    sprites: &mut [Sprite],
    gate_pos: (f32, f32),
    got_coins: usize,
    sm: &mut SpriteManager,
    block_size: usize,
    dist_plane: f32,
    zbuf: &[f32],
) {
    // Construir lista: monedas vivas + sprite del portal (polis/quiz)
    let mut all = Vec::new();
    all.extend(sprites.iter().filter(|s| s.alive).cloned());
    all.push(Sprite {
        x: gate_pos.0,
        y: gate_pos.1,
        size: block_size as f32 * 1.0,
        ch: if got_coins >= 3 { 'Q' } else { 'P' },
        alive: true,
    });

    // Ordenar de lejos a cerca (pintamos del más lejano al más cercano)
    all.sort_by(|a, b| {
        let da = (a.x - player.pos.x).hypot(a.y - player.pos.y);
        let db = (b.x - player.pos.x).hypot(b.y - player.pos.y);
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });

    let hw = framebuffer.width as f32 * 0.5;
    let hh = framebuffer.height as f32 * 0.5;

    for s in all {
        // Vector del jugador al sprite (mundo)
        let dx = s.x - player.pos.x;
        let dy = s.y - player.pos.y;

        // Ángulo absoluto al sprite y diferencia relativa al ángulo de la cámara
        let sprite_ang = dy.atan2(dx);
        let mut diff = normalize_angle(sprite_ang - player.a);

        // Culling por FOV (con margen pequeño)
        let fov_half = player.fov * 0.5;
        if diff.abs() > fov_half * 1.05 { continue; }

        // Distancia euclidiana y distancia perpendicular al plano de proyección
        let dist = dx.hypot(dy);
        let mut perp = (dist * diff.cos().abs()).max(0.0001);
        let near = 0.30 * block_size as f32;
        if perp < near { perp = near; }

        // Proyección billboard estable
        let sprite_h = (s.size * dist_plane) / perp;
        let sprite_w = sprite_h; // cuadrado
        let screen_x = hw + diff.tan() * dist_plane;

        let x0 = (screen_x - sprite_w * 0.5).floor() as i32;
        let x1 = (screen_x + sprite_w * 0.5).ceil()  as i32;
        let y0 = (hh - sprite_h * 0.5).floor() as i32;
        let y1 = (hh + sprite_h * 0.5).ceil()  as i32;

        // Tamaño de textura
        let (tw_u, th_u) = sm.get_size(s.ch);
        let tw = tw_u as i32; let th = th_u as i32;
        if tw <= 0 || th <= 0 { continue; }

        // Raster columna a columna
        for sx in x0.max(0) ..= x1.min(framebuffer.width as i32 - 1) {
            let col = sx as usize;

            // Oclusión: si pared está delante de este sprite en esta columna, saltar
            if col < zbuf.len() && perp >= zbuf[col] - 0.001 { continue; }

            let u = (sx as f32 - (screen_x - sprite_w * 0.5)) / sprite_w; // [0,1]
            if !(0.0..=1.0).contains(&u) { continue; }
            let tex_x = (u * tw as f32).clamp(0.0, (tw - 1) as f32) as i32;

            let yy0 = y0.max(0);
            let yy1 = y1.min(framebuffer.height as i32 - 1);
            for sy in yy0 ..= yy1 {
                let v = (sy as f32 - y0 as f32) / ((y1 - y0).max(1) as f32);
                if !(0.0..=1.0).contains(&v) { continue; }
                let tex_y = (v * th as f32).clamp(0.0, (th - 1) as f32) as i32;

                let c = sm.sample(s.ch, tex_x as u32, tex_y as u32);
                // Transparencia: alpha o color-key opcional
                if c.a < 16 { continue; }
                if c.r == 152 && c.g == 0 && c.b == 136 && c.a == 255 { continue; }

                framebuffer.set_current_color(c);
                framebuffer.set_pixel(sx as u32, sy as u32);
            }
        }
    }
}

/// Check pickup: si el jugador está cerca de una moneda -> recolectar
pub fn pickup_coins(player: &Player, sprites: &mut [Sprite], block_size: usize) -> usize {
    let r = 0.35 * block_size as f32;
    let mut count = 0;
    for s in sprites.iter_mut() {
        if s.alive && s.ch == 'c' {
            let d = (s.x - player.pos.x).hypot(s.y - player.pos.y);
            if d < r { s.alive = false; }
        }
        if s.alive == false && s.ch == 'c' {
            // contar no, solo vivos; contamos afuera
        }
    }
    for s in sprites.iter() {
        if s.alive && s.ch == 'c' { count += 1; }
    }
    let collected = 3usize.saturating_sub(count);
    collected
}