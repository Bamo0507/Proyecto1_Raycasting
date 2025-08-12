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
use std::time::{Instant};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum GameState { Welcome, LevelPicker, Playing1, Playing2, Lost, Win }

struct Screens {
    welcome: Texture2D,
    picker:  Texture2D,
    lost:    Texture2D,
    win:     Texture2D,
}

fn cell_to_color(cell: char) -> Color {
    match cell {
        '+' => Color::new(90, 140, 255, 255),   // azul eléctrico (más visible)
        '-' => Color::new(0, 218, 209, 255),    // cian vibrante
        '|' => Color::new(255, 120, 190, 255),  // rosa brillante
        'g' => Color::new(255, 219, 88, 255),   // dorado cálido
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
    let pad = 1; // separación de 1px entre celdas
    framebuffer.fill_rect(
        x0 + pad,
        y0 + pad,
        (block_size as i32 - 2 * pad).max(1),
        (block_size as i32 - 2 * pad).max(1),
    );
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player, coins: &Vec<Sprite>, origin: (i32, i32), scale: f32) {
    let mini = ((block_size as f32) * scale) as usize;
    let (ox, oy) = origin;

    let mw = (maze[0].len() * mini) as i32;
    let mh = (maze.len() * mini) as i32;
    // fondo del minimapa (teal más claro)
    framebuffer.set_current_color(Color::new(60, 190, 180, 230));
    framebuffer.fill_rect(ox, oy, mw, mh);
    // borde fino (navy)
    framebuffer.set_current_color(Color::new(24, 32, 56, 255));
    framebuffer.fill_rect(ox - 1, oy - 1, mw + 2, 1);
    framebuffer.fill_rect(ox - 1, oy + mh, mw + 2, 1);
    framebuffer.fill_rect(ox - 1, oy - 1, 1, mh + 2);
    framebuffer.fill_rect(ox + mw, oy - 1, 1, mh + 2);

    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * mini;
            let yo = row_index * mini;
            draw_cell_at(framebuffer, ox, oy, xo, yo, mini, cell);
        }
    }

    // monedas en el minimapa (vivas)
    for s in coins.iter().filter(|s| s.is_active && s.sprite_type == 'c') {
        let cx = (s.world_x / block_size as f32) * mini as f32;
        let cy = (s.world_y / block_size as f32) * mini as f32;
        framebuffer.set_current_color(Color::GOLD);
        framebuffer.fill_rect(ox + cx as i32 - 2, oy + cy as i32 - 2, 4, 4);
    }

    // jugador en el minimapa (posición subcelda para movimiento suave)
    let pmini_x = (player.position.x / block_size as f32) * mini as f32;
    let pmini_y = (player.position.y / block_size as f32) * mini as f32;
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

    // Cielo en degradado (celeste claro -> blanco azulado)
    let sky_top = Color::new(179, 229, 252, 255);   // celeste claro
    let sky_bot = Color::new(224, 247, 250, 255);   // blanco azulado
    let sky_h = hh as i32;
    for y in 0..sky_h {
        let t = (y as f32) / (sky_h as f32).max(1.0);
        let r = (sky_top.r as f32 * (1.0 - t) + sky_bot.r as f32 * t) as u8;
        let g = (sky_top.g as f32 * (1.0 - t) + sky_bot.g as f32 * t) as u8;
        let b = (sky_top.b as f32 * (1.0 - t) + sky_bot.b as f32 * t) as u8;
        framebuffer.set_current_color(Color::new(r, g, b, 255));
        framebuffer.fill_rect(0, y, framebuffer.width as i32, 1);
    }

    // Piso: teal más oscuro
    framebuffer.set_current_color(Color::new(26, 120, 112, 255));
    framebuffer.fill_rect(0, hh as i32, framebuffer.width as i32, hh as i32);

    // Distancia del plano de proyección (corrección por FOV)
    let dist_plane = (framebuffer.width as f32 / 2.0) / (player.field_of_view / 2.0).tan();
    let bs = block_size as f32;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32; // [0,1)
        let a = player.angle - (player.field_of_view / 2.0) + (player.field_of_view * current_ray);
        let hit = cast_ray(framebuffer, maze, player, a, block_size, false);

        // Corrección de "fisheye": distancia perpendicular
        let mut perp = hit.hit_distance * (a - player.angle).cos().abs();
        // Near plane más agresivo para evitar columnas gigantes al acercarse
        let near = 0.35 * bs; // 35% del tamaño de la celda
        if perp < near { perp = near; }
        zbuf[i as usize] = perp;

        // Altura de la columna (stake) con límite superior más estricto
        let stake_height = ((bs * dist_plane) / perp).min(framebuffer.height as f32 * 0.9);
        let stake_top = (hh - stake_height * 0.5) as i32;
        let stake_bottom = (hh + stake_height * 0.5) as i32;

        // Dimensiones de la textura para el tipo de pared impactada
        let (tw_u, th_u) = texman.get_image_size(hit.wall_type);
        let tw = tw_u as i32; let th = th_u as i32;

        // Coordenada X dentro de la textura usando fracción robusta provista por el raycast
        let tex_x = (hit.texture_coord * tw as f32).clamp(0.0, tw as f32 - 1.0) as i32;

        // Pintar la columna muestreando la textura y sombreando por distancia
        let y_start = stake_top.max(0);
        let y_end = stake_bottom.min(framebuffer.height as i32 - 1);
        for y in y_start..=y_end {
            let v = (y as f32 - y_start as f32) / ((y_end - y_start).max(1) as f32);
            let tex_y = (v * th as f32).clamp(0.0, th as f32 - 1.0) as i32;

            let c = texman.get_pixel_color_mut(hit.wall_type, tex_x as u32, tex_y as u32);
            framebuffer.set_current_color(c);
            framebuffer.set_pixel(i, y as u32);
        }
    }
    zbuf
}

fn draw_fullscreen_screen(
    window: &mut RaylibHandle,
    thread: &RaylibThread,
    tex: &Texture2D,
    fade_t: f32, // 0=no fade overlay, 1=negro total
) {
    let mut d = window.begin_drawing(thread);
    d.clear_background(Color::BLACK);
    d.draw_texture(tex, 0, 0, Color::WHITE);

    if fade_t > 0.0 {
        let a = (fade_t.clamp(0.0, 1.0) * 255.0) as u8;
        d.draw_rectangle(0, 0, d.get_screen_width(), d.get_screen_height(), Color::new(0,0,0,a));
    }
}

fn start_level(
    which: usize,
    block_size: usize,
    player: &mut Player,
    maze_out: &mut Maze,
    coins_out: &mut Vec<Sprite>,
    keys_total_out: &mut usize,
) -> (/*gate_pos*/(f32,f32), /*level_start*/Instant)
{
    let maze_file = if which == 1 { "maze1.txt" } else { "maze2.txt" };
    *maze_out = load_maze(maze_file);
    *keys_total_out = if which == 1 { 5 } else { 8 };

    // posicion jugador en 'p'
    let (si, sj) = find_char(&maze_out, 'p').unwrap_or((1, 1));
    let sx = (si * block_size + block_size / 2) as f32;
    let sy = (sj * block_size + block_size / 2) as f32;
    player.position = Vector2::new(sx, sy);

    // monedas
    *coins_out = spawn_coins(&maze_out, block_size, *keys_total_out);

    // gate 'g'
    let (gi, gj) = find_char(&maze_out, 'g').unwrap_or((maze_out[0].len()-2, maze_out.len()-2));
    let gate_pos = (
        (gi * block_size + block_size/2) as f32,
        (gj * block_size + block_size/2) as f32
    );

    (gate_pos, Instant::now())
}

fn pressed_enter(win: &RaylibHandle) -> bool {
    win.is_key_pressed(KeyboardKey::KEY_ENTER)
        || win.is_key_pressed(KeyboardKey::KEY_KP_ENTER)
        || win.is_key_pressed(KeyboardKey::KEY_SPACE)
}
fn pressed_one(win: &RaylibHandle) -> bool {
    win.is_key_pressed(KeyboardKey::KEY_ONE) || win.is_key_pressed(KeyboardKey::KEY_KP_1)
}
fn pressed_two(win: &RaylibHandle) -> bool {
    win.is_key_pressed(KeyboardKey::KEY_TWO) || win.is_key_pressed(KeyboardKey::KEY_KP_2)
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
        .title("Subway Surfer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let screens = Screens {
        welcome: window.load_texture(&raylib_thread, "assets/welcome_screen.png").expect("welcome_screen"),
        picker:  window.load_texture(&raylib_thread, "assets/level_picker.png").expect("level_picker"),
        lost:    window.load_texture(&raylib_thread, "assets/lost.png").expect("lost"),
        win:     window.load_texture(&raylib_thread, "assets/win.png").expect("win"),
    };

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
    let coin_snd   = audio.new_sound("assets/coin.mp3").expect("No se pudo cargar assets/coin.mp3");
    let key_snd    = audio.new_sound("assets/key.mp3").expect("No se pudo cargar assets/key.mp3");

    let mut maze: Maze = Vec::new();

    let mut state = GameState::Welcome;

    // fade: 0.0 (opaco) → 1.0 (desvanecido)
    let mut fading = false;
    let mut fade_t: f32 = 0.0;           // 0..1
    let fade_speed: f32 = 3.0;           // ajusta velocidad del fade
    let mut fade_dir: f32 = 0.0;         // +1 = fade-out, -1 = fade-in
    let mut next_state: Option<GameState> = None;

    // nivel y llaves por nivel
    let mut keys_total: usize = 5;// se setea al entrar a cada nivel

    // Cargar texturas de paredes (wall + graffiti por tipo)
    let mut texman = TextureManager::new(&mut window, &raylib_thread).expect("Error cargando texturas");

    let mut player = Player {
        position: Vector2::new((block_size / 2) as f32, (block_size / 2) as f32), // placeholder
        angle: PI / 3.0,
        field_of_view: PI / 2.0,
    };

    // Cargar sprites
    let mut spriteman = SpriteMgr::new().expect("Error cargando sprites");

    // Lista de monedas (se crea al seleccionar nivel) y posición de portal
    let mut coins: Vec<Sprite> = Vec::new();
    let mut gate_pos: (f32, f32) = (0.0, 0.0);

    // Contador
    let mut _collected = 0usize;

    // Timer de nivel y estado de audio/proximidad
    let mut level_total = Duration::from_secs(60);
    let mut level_start = Instant::now();
    let mut lost = false;
    let mut last_police = Instant::now() - Duration::from_millis(1000);
    let mut prev_collected = 0usize;
    let mut _picked_key = false;

    // Main render loop
    while !window.window_should_close() {
        // Música
        theme.update_stream();

        // Fade step (global)
        if fading {
            let dt = window.get_frame_time();
            fade_t = (fade_t + fade_dir * fade_speed * dt).clamp(0.0, 1.0);
            if fade_dir > 0.0 && fade_t >= 1.0 {
                if let Some(ns) = next_state.take() { state = ns; }
                fade_dir = -1.0; // begin fade-in on the new screen
            } else if fade_dir < 0.0 && fade_t <= 0.0 {
                fading = false; fade_dir = 0.0; fade_t = 0.0;
            }
        }

        match state {
    GameState::Welcome => {
        draw_fullscreen_screen(&mut window, &raylib_thread, &screens.welcome, if fading { fade_t } else { 0.0 });

        if pressed_enter(&window) && !fading {
            println!("[state] Welcome -> LevelPicker (fade)");
            fading = true; fade_dir = 1.0; fade_t = 0.0; next_state = Some(GameState::LevelPicker);
        }
        continue;
    }

    GameState::LevelPicker => {
        draw_fullscreen_screen(&mut window, &raylib_thread, &screens.picker, if fading { fade_t } else { 0.0 });

        if pressed_one(&window) {
            let (gp, st) = start_level(1, block_size, &mut player, &mut maze, &mut coins, &mut keys_total);
            gate_pos = gp; level_start = st;
            _collected = 0; lost = false; prev_collected = 0; _picked_key = false;
            println!("[state] Start Level 1 | keys_total={} | coins={} cells", keys_total, coins.len());
            if !fading { fading = true; fade_dir = 1.0; fade_t = 0.0; next_state = Some(GameState::Playing1); }
            continue;
        }
        if pressed_two(&window) {
            let (gp, st) = start_level(2, block_size, &mut player, &mut maze, &mut coins, &mut keys_total);
            gate_pos = gp; level_start = st;
            _collected = 0; lost = false; prev_collected = 0; _picked_key = false;
            println!("[state] Start Level 2 | keys_total={} | coins={} cells", keys_total, coins.len());
            if !fading { fading = true; fade_dir = 1.0; fade_t = 0.0; next_state = Some(GameState::Playing2); }
            continue;
        }
        if window.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            println!("[state] LevelPicker -> Welcome (fade)");
            if !fading { fading = true; fade_dir = 1.0; fade_t = 0.0; next_state = Some(GameState::Welcome); }
            continue;
        }
        continue;
    }

    GameState::Playing1 | GameState::Playing2 => {
        // === tu bucle de juego actual ===
        framebuffer.clear();

        process_events(&mut player, &window, &maze, block_size);

        let _collected = pickup_coins(&player, &mut coins, block_size);

        // Sonido y bonus de tiempo al recoger nuevas coins ( +5s por coin )
        if _collected > prev_collected {
            let gained = (_collected - prev_collected) as u64;
            coin_snd.play();
            level_total += Duration::from_secs(5 * gained);
            prev_collected = _collected;
        }

        let elapsed = level_start.elapsed();
        let time_left = if elapsed >= level_total { 0 } else { (level_total - elapsed).as_secs() as u32 };
        if time_left == 0 && !lost {
            lost = true; police_snd.play();
            println!("[state] Time up -> Lost");
            state = GameState::Lost;
        }

        if !lost && _collected < keys_total {
            let dx = player.position.x - gate_pos.0;
            let dy = player.position.y - gate_pos.1;
            let d = (dx*dx + dy*dy).sqrt();
            if d < (block_size as f32 * 1.1) && last_police.elapsed() > Duration::from_millis(800) {
                police_snd.play(); last_police = Instant::now();
            }
        }

        // mundo, sprites, minimapa, HUD
        let zbuf = render_world(&mut framebuffer, &maze, block_size, &player, &mut texman);
        let dist_plane = (framebuffer.width as f32 / 2.0) / (player.field_of_view / 2.0).tan();
        render_sprites(&mut framebuffer, &player, &mut coins, gate_pos, _collected, keys_total, &mut spriteman, block_size, dist_plane, &zbuf);
        render_minimap(&mut framebuffer, &maze, block_size, &player, &coins, (16, 16), 0.15);
        framebuffer.swap_buffers(&mut window, &raylib_thread, _collected, keys_total, time_left);

        // ¿ganaste? (tocar la key en la puerta con 5/8)
        if _collected >= keys_total {
            let dx = player.position.x - gate_pos.0;
            let dy = player.position.y - gate_pos.1;
            let d = (dx*dx + dy*dy).sqrt();
            if d < (block_size as f32 * 0.7) {
                key_snd.play();
                println!("[state] Gate reached with all keys -> Win");
                state = GameState::Win;
            }
        }
        continue;
    }

    GameState::Lost => {
        draw_fullscreen_screen(&mut window, &raylib_thread, &screens.lost, if fading { fade_t } else { 0.0 });
        if pressed_enter(&window) {
            println!("[state] Lost -> LevelPicker (fade)");
            if !fading { fading = true; fade_dir = 1.0; fade_t = 0.0; next_state = Some(GameState::LevelPicker); }
        }
        continue;
    }

    GameState::Win => {
        draw_fullscreen_screen(&mut window, &raylib_thread, &screens.win, if fading { fade_t } else { 0.0 });
        if pressed_enter(&window) {
            println!("[state] Win -> Welcome (fade)");
            if !fading { fading = true; fade_dir = 1.0; fade_t = 0.0; next_state = Some(GameState::Welcome); }
        }
        continue;
    }
}
    }
}