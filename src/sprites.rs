use std::f32::consts::PI;
use raylib::prelude::*;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::rng;
use image::ImageReader;
use crate::maze::{Maze, is_walkable};
use crate::player::Player;
use crate::framebuffer::Framebuffer;

struct CpuImage {
    width: u32,
    height: u32,
    pixel_data: Vec<u8>, 
}

fn normalize_angle(mut angle_radians: f32) -> f32 {
    while angle_radians >  PI { angle_radians -= 2.0 * PI; }
    while angle_radians < -PI { angle_radians += 2.0 * PI; }
    angle_radians
}

fn load_any_image_rgba8(path: &str) -> Result<CpuImage, String> {
    let reader = ImageReader::open(path)
        .map_err(|e| format!("open {}: {}", path, e))?
        .with_guessed_format()
        .map_err(|e| format!("format {}: {}", path, e))?;
    let dynimg = reader.decode().map_err(|e| format!("decode {}: {}", path, e))?;
    let rgba = dynimg.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(CpuImage { width: w, height: h, pixel_data: rgba.into_raw() })
}

#[derive(Clone)]
pub struct Sprite {
    pub world_x: f32, // Posición X en el mundo (píxeles)
    pub world_y: f32, // Posición Y en el mundo (píxeles)
    pub size: f32, // Tamaño en píxeles del mundo (antes de proyección)
    pub sprite_type: char, // Tipo de sprite: 'c' = moneda, 'P' = policía, 'Q' = portal
    pub is_active: bool, // Si el sprite está activo/visible
}

pub struct SpriteManager {
    sprite_textures: HashMap<char, CpuImage>, // Texturas de sprites cargadas
}

impl SpriteManager {
    pub fn new() -> Result<Self, String> {
        let mut sprite_textures = HashMap::new();

        // Cargar sprites
        let coin = load_any_image_rgba8("assets/coin.webp")?;
        let police = load_any_image_rgba8("assets/police.png")?;
        let keys = load_any_image_rgba8("assets/keys.webp")?;

        sprite_textures.insert('c', coin);
        sprite_textures.insert('P', police);
        sprite_textures.insert('Q', keys);

        Ok(Self { sprite_textures })
    }

    pub fn get_size(&self, sprite_type: char) -> (u32, u32) {
        if let Some(img) = self.sprite_textures.get(&sprite_type) {
            (img.width, img.height)
        } else {
            (1, 1)
        }
    }

    pub fn sample(&self, sprite_type: char, texture_x: u32, texture_y: u32) -> Color {
        if let Some(img) = self.sprite_textures.get(&sprite_type) {
            let width = img.width.max(1);
            let height = img.height.max(1);
            let x = texture_x.min(width - 1);
            let y = texture_y.min(height - 1);
            let pixel_index = ((y * width + x) as usize) * 4;
            if pixel_index + 3 < img.pixel_data.len() {
                let r = img.pixel_data[pixel_index];
                let g = img.pixel_data[pixel_index + 1];
                let b = img.pixel_data[pixel_index + 2];
                let a = img.pixel_data[pixel_index + 3];
                return Color::new(r, g, b, a);
            }
        }
        Color::WHITE
    }
}

/// Selecciona `n` celdas libres al azar (no son paredes) y crea sprites “moneda”
pub fn spawn_coins(maze: &Maze, block_size: usize, n: usize) -> Vec<Sprite> {
    let mut free_cells = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if is_walkable(c) {
                free_cells.push((i, j));
            }
        }
    }
    free_cells.shuffle(&mut rng());

    let mut out = Vec::new();
    for (i, j) in free_cells.into_iter().take(n) {
        let x = (i * block_size + block_size / 2) as f32;
        let y = (j * block_size + block_size / 2) as f32;
        out.push(Sprite {
            world_x: x,
            world_y: y,
            size: (block_size as f32) * 0.4, // moneda más chica (40% de la celda)
            sprite_type: 'c',
            is_active: true,
        });
    }
    out
}


pub fn render_sprites(
    framebuffer: &mut Framebuffer,
    player: &Player,
    sprites: &mut [Sprite],
    portal_position: (f32, f32),
    coins_collected: usize,
    coins_needed: usize,
    sprite_manager: &mut SpriteManager,
    block_size: usize,
    projection_distance: f32,
    depth_buffer: &[f32],
) {
    // Construir lista: monedas vivas + sprite de cierre nivel (police/keys)
    let mut visible_sprites = Vec::new();
    visible_sprites.extend(sprites.iter().filter(|s| s.is_active).cloned());
    visible_sprites.push(Sprite {
        world_x: portal_position.0,
        world_y: portal_position.1,
        size: block_size as f32 * 1.0,
        sprite_type: if coins_collected >= coins_needed { 'Q' } else { 'P' },
        is_active: true,
    });

    // Ordenar de lejos a cerca 
    visible_sprites.sort_by(|a, b| {
        let distance_a = (a.world_x - player.position.x).hypot(a.world_y - player.position.y);
        let distance_b = (b.world_x - player.position.x).hypot(b.world_y - player.position.y);
        distance_b.partial_cmp(&distance_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let half_width = framebuffer.width as f32 * 0.5;
    let half_height = framebuffer.height as f32 * 0.5;

    for sprite in visible_sprites {
        // Vector del jugador al sprite
        let dx = sprite.world_x - player.position.x;
        let dy = sprite.world_y - player.position.y;

        let sprite_angle = dy.atan2(dx);
        let diff = normalize_angle(sprite_angle - player.angle);

        // Culling por FOV
        let fov_half = player.field_of_view * 0.5;
        if diff.abs() > fov_half * 1.05 { continue; }

        let distance = dx.hypot(dy);
        let mut perp = (distance * diff.cos().abs()).max(0.0001);
        let near = 0.30 * block_size as f32;
        if perp < near { perp = near; }

        let sprite_height = (sprite.size * projection_distance) / perp;
        let sprite_width = sprite_height; 
        let screen_x = half_width + diff.tan() * projection_distance;

        let x0 = (screen_x - sprite_width * 0.5).floor() as i32;
        let x1 = (screen_x + sprite_width * 0.5).ceil()  as i32;
        let y0 = (half_height - sprite_height * 0.5).floor() as i32;
        let y1 = (half_height + sprite_height * 0.5).ceil()  as i32;

        // Tamaño de textura
        let (texture_width, texture_height) = sprite_manager.get_size(sprite.sprite_type);
        let texture_width = texture_width as i32; let texture_height = texture_height as i32;
        if texture_width <= 0 || texture_height <= 0 { continue; }

        // Raster columna a columna
        for sx in x0.max(0) ..= x1.min(framebuffer.width as i32 - 1) {
            let col = sx as usize;

            // Oclusión: si pared está delante de este sprite en esta columna, saltar
            if col < depth_buffer.len() && perp >= depth_buffer[col] - 0.001 { continue; }

            let u = (sx as f32 - (screen_x - sprite_width * 0.5)) / sprite_width; // [0,1]
            if !(0.0..=1.0).contains(&u) { continue; }
            let texture_x = (u * texture_width as f32).clamp(0.0, (texture_width - 1) as f32) as i32;

            let yy0 = y0.max(0);
            let yy1 = y1.min(framebuffer.height as i32 - 1);
            for sy in yy0 ..= yy1 {
                let v = (sy as f32 - y0 as f32) / ((y1 - y0).max(1) as f32);
                if !(0.0..=1.0).contains(&v) { continue; }
                let texture_y = (v * texture_height as f32).clamp(0.0, (texture_height - 1) as f32) as i32;

                let c = sprite_manager.sample(sprite.sprite_type, texture_x as u32, texture_y as u32);
                // Transparencia: alpha o color-key opcional
                if c.a < 16 { continue; }
                if c.r == 152 && c.g == 0 && c.b == 136 && c.a == 255 { continue; }

                framebuffer.set_current_color(c);
                framebuffer.set_pixel(sx as u32, sy as u32);
            }
        }
    }
}

/// Verifica y recoge monedas cercanas al jugador
pub fn pickup_coins(player: &Player, sprites: &mut [Sprite], block_size: usize) -> usize {
    let pickup_radius = 0.35 * block_size as f32;

    for sprite in sprites.iter_mut() {
        if sprite.is_active && sprite.sprite_type == 'c' {
            let distance_to_player = (sprite.world_x - player.position.x)
                .hypot(sprite.world_y - player.position.y);
            if distance_to_player < pickup_radius { 
                sprite.is_active = false; 
            }
        }
    }

    // Conteo de monedas
    let total_coins = sprites.iter().filter(|s| s.sprite_type == 'c').count();
    let remaining_coins = sprites.iter()
        .filter(|s| s.sprite_type == 'c' && s.is_active)
        .count();
    
    total_coins.saturating_sub(remaining_coins)
}