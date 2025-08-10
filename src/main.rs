mod framebuffer;
mod maze;
mod caster;
mod player;
mod textures;
mod sprites;

use maze::{Maze, load_maze, find_char};
use caster::cast_ray;
use framebuffer::Framebuffer;
use player::{Player, process_events};
use raylib::prelude::*;
use crate::textures::TextureManager;
use sprites::{SpriteManager as SpriteMgr, spawn_coins, render_sprites, pickup_coins, Sprite};
use raylib::core::audio::{RaylibAudio, Sound, Music};
use std::time::{Duration, Instant};

const KEYS_TOTAL: usize = 5;

fn cell_to_color(cell: char) -> Color {
    match cell {
        '+' => Color::new(24, 32, 56, 255),   // navy oscuro
        '-' => Color::new(0, 218, 209, 255),  // cian vibrante
        '|' => Color::new(245, 96, 170, 255), // rosa fuerte
        'g' => Color::new(255, 219, 88, 255), // dorado cálido
        _ => Color::LIGHTGRAY,
    }
}

fn draw_cell_at(
    framebuffer: &mut Framebuffer,
    ox: i32,
    oy: i32,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    if cell == ' ' { return; }
    let color = cell_to_color(cell);
    framebuffer.set_current_color(color);
    let x0 = ox + xo as i32;
    let y0 = oy + yo as i32;
    framebuffer.fill_rect(x0, y0, block_size as i32, block_size as i32);
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player, coins: &Vec<Sprite>, origin: (i32, i32), scale: f32) {
    let mini = ((block_size as f32) * scale) as usize;
    let (ox, oy) = origin;

    // fondo del minimapa (área completa, color visible pero semitransparente)
    framebuffer.set_current_color(Color::new(235, 192, 121, 255));
    framebuffer.fill_rect(ox, oy, (maze[0].len() * mini) as i32, (maze.len() * mini) as i32);

    // marco del minimapa
    framebuffer.set_current_color(Color::new(235, 192, 121, 255));
    framebuffer.fill_rect(ox - 4, oy - 4, (maze[0].len() * mini + 8) as i32, (maze.len() * mini + 8) as i32);

    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * mini;
            let yo = row_index * mini;
            draw_cell_at(framebuffer, ox, oy, xo, yo, mini, cell);
        }
    }

    // monedas en el minimapa (vivas)
    for s in coins.iter().filter(|s| s.alive && s.ch == 'c') {
        let cx = (s.x / block_size as f32) * mini as f32;
        let cy = (s.y / block_size as f32) * mini as f32;
        framebuffer.set_current_color(Color::GOLD);
        framebuffer.fill_rect(ox + cx as i32 - 2, oy + cy as i32 - 2, 4, 4);
    }

    // jugador en el minimapa (posición subcelda para movimiento suave)
    let pmini_x = (player.pos.x / block_size as f32) * mini as f32;
    let pmini_y = (player.pos.y / block_size as f32) * mini as f32;
    framebuffer.set_current_color(Color::BLACK);
    framebuffer.fill_rect(ox + pmini_x as i32 - 2, oy + pmini_y as i32 - 2, 5, 5);
}

fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    texman: &mut TextureManager,
) -> Vec<f32> {
    let num_rays = framebuffer.width; // 1 rayo por columna
    let hh = framebuffer.height as f32 / 2.0; // half height
    let mut zbuf = vec![f32::INFINITY; framebuffer.width as usize];

    // Cielo (techo frío)
    framebuffer.set_current_color(Color::SKYBLUE);
    framebuffer.fill_rect(0, 0, framebuffer.width as i32, hh as i32);
    // Piso (floor color from sampled image: R=215, G=214, B=182)
    framebuffer.set_current_color(Color::new(215, 214, 182, 255));
    framebuffer.fill_rect(0, hh as i32, framebuffer.width as i32, hh as i32);

    // Distancia del plano de proyección (corrección por FOV)
    let dist_plane = (framebuffer.width as f32 / 2.0) / (player.fov / 2.0).tan();
    let bs = block_size as f32;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32; // [0,1)
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let hit = cast_ray(framebuffer, maze, player, a, block_size, false);

        // Corrección de "fisheye": distancia perpendicular
        let mut perp = hit.distance * (a - player.a).cos().abs();
        // Near plane más agresivo para evitar columnas gigantes al acercarse
        let near = 0.35 * bs; // 35% del tamaño de la celda
        if perp < near { perp = near; }
        zbuf[i as usize] = perp;

        // Altura de la columna (stake) con límite superior más estricto
        let stake_height = ((bs * dist_plane) / perp).min(framebuffer.height as f32 * 0.9);
        let stake_top = (hh - stake_height * 0.5) as i32;
        let stake_bottom = (hh + stake_height * 0.5) as i32;

        // Dimensiones de la textura para el tipo de pared impactada
        let (tw_u, th_u) = texman.get_image_size(hit.impact);
        let tw = tw_u as i32; let th = th_u as i32;

        // Coordenada X dentro de la textura usando fracción robusta provista por el raycast
        let tex_x = (hit.tex_frac * tw as f32).clamp(0.0, tw as f32 - 1.0) as i32;

        // Pintar la columna muestreando la textura y sombreando por distancia
        let y_start = stake_top.max(0);
        let y_end = stake_bottom.min(framebuffer.height as i32 - 1);
        for y in y_start..=y_end {
            let v = (y as f32 - y_start as f32) / ((y_end - y_start).max(1) as f32);
            let tex_y = (v * th as f32).clamp(0.0, th as f32 - 1.0) as i32;

            let c = texman.get_pixel_color_mut(hit.impact, tex_x as u32, tex_y as u32);
            framebuffer.set_current_color(c);
            framebuffer.set_pixel(i, y as u32);
        }
    }
    zbuf
}

pub fn main() {
    use raylib::prelude::*;
    use std::f32::consts::PI;
    use std::time::Duration;

    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster 3D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Color::new(50, 50, 100, 255));

    // --- Audio (raylib-rs 5.5.1) ---
    let audio = RaylibAudio::init_audio_device().expect("No se pudo iniciar el audio");

    // Música de fondo (stream)
    let theme = audio.new_music("assets/theme.mp3").expect("No se pudo cargar assets/theme.mp3");
    theme.set_volume(0.6);
    theme.play_stream();

    // SFX
    let police_snd = audio.new_sound("assets/police.mp3").expect("No se pudo cargar assets/police.mp3");
    let key_snd    = audio.new_sound("assets/key.mp3").expect("No se pudo cargar assets/key.mp3");

    let maze = load_maze("maze.txt");

    // Cargar texturas de paredes (wall + graffiti por tipo)
    let mut texman = TextureManager::new(&mut window, &raylib_thread).expect("Error cargando texturas");

    // Posición inicial en 'p' (centro del bloque)
    let (start_i, start_j) = find_char(&maze, 'p').unwrap_or((1, 1));
    let start_x = (start_i * block_size + block_size / 2) as f32;
    let start_y = (start_j * block_size + block_size / 2) as f32;

    let mut player = Player {
        pos: Vector2::new(start_x, start_y),
        a: PI / 3.0,
        fov: PI / 2.0,
    };

    // Cargar sprites
    let mut spriteman = SpriteMgr::new().expect("Error cargando sprites");

    // Posiciones aleatorias de 3 monedas
    let mut coins: Vec<Sprite> = spawn_coins(&maze, block_size, KEYS_TOTAL);

    // Posición del portal 'g' (centro de la celda)
    let (gi, gj) = find_char(&maze, 'g').unwrap_or((maze[0].len()-2, maze.len()-2));
    let gate_pos = (
        (gi * block_size + block_size/2) as f32,
        (gj * block_size + block_size/2) as f32
    );

    // Contador
    let mut collected = 0usize;

    // Timer de nivel y estado de audio/proximidad
    let level_total = Duration::from_secs(60);
    let level_start = Instant::now();
    let mut lost = false;
    let mut last_police = Instant::now() - Duration::from_millis(1000);
    let mut prev_collected = 0usize;
    let mut picked_key = false;

    // Main render loop
    while !window.window_should_close() {
        // Resetea el lienzo
        framebuffer.clear();
        theme.update_stream();

        // actua acorde a inputs o colisiones
        process_events(&mut player, &window, &maze, block_size);

        collected = pickup_coins(&player, &mut coins, block_size);

        // Sonido de coin al recoger una nueva
        if collected > prev_collected {
            key_snd.play();
            prev_collected = collected;
        }

        // Timer de nivel
        let elapsed = level_start.elapsed();
        let time_left = if elapsed >= level_total { 0 } else { (level_total - elapsed).as_secs() as u32 };

        // Si se acabó el tiempo, marcar derrota y sonar policía una vez
        if time_left == 0 && !lost {
            lost = true;
            police_snd.play();
        }

        // Proximidad al portal bloqueado: sirena si te acercas sin las 3 coins (con cooldown)
        if !lost && collected < KEYS_TOTAL {
            let dx = player.pos.x - gate_pos.0;
            let dy = player.pos.y - gate_pos.1;
            let d = (dx*dx + dy*dy).sqrt();
            if d < (block_size as f32 * 1.1) && last_police.elapsed() > Duration::from_millis(800) {
                police_snd.play();
                last_police = Instant::now();
            }
        }

        // Si ya tienes todas las monedas, al tocar la llave (en la puerta) suena key.mp3 una sola vez
        if !lost && !picked_key && collected >= KEYS_TOTAL {
            let dx = player.pos.x - gate_pos.0;
            let dy = player.pos.y - gate_pos.1;
            let d = (dx*dx + dy*dy).sqrt();
            if d < (block_size as f32 * 0.7) {
                key_snd.play();
                picked_key = true; // evita repetir sonido
            }
        }

        // RENDER STEP
        // renderiza el mundo 3D y obtiene z-buffer
        let zbuf = render_world(&mut framebuffer, &maze, block_size, &player, &mut texman);

        // renderiza sprites (monedas + portal bloqueado/abierto)
        let dist_plane = (framebuffer.width as f32 / 2.0) / (player.fov / 2.0).tan();
        render_sprites(
            &mut framebuffer,
            &player,
            &mut coins,
            gate_pos,
            collected,
            &mut spriteman,
            block_size,
            dist_plane,
            &zbuf,
        );

        // renderiza el minimapa
        render_minimap(&mut framebuffer, &maze, block_size, &player, &coins, (16, 16), 0.15);

        // Ya pasamos del Image al Texture2D, ahora se encarga de dibujar el Texture2D en la ventana
        framebuffer.swap_buffers(&mut window, &raylib_thread, collected, KEYS_TOTAL, time_left, lost);

        // Tasa de refresco
        std::thread::sleep(Duration::from_millis(16));
    }
}