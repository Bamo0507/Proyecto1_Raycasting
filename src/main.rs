mod framebuffer;
mod maze;
mod caster;
mod player;
use maze::{Maze, load_maze, find_char};
use caster::cast_ray;
use framebuffer::Framebuffer;
use player::{Player, process_events};
use raylib::prelude::*;

fn cell_to_color(cell: char) -> Color {
    match cell {
        '+' => Color::BLUEVIOLET,
        '-' => Color::VIOLET,
        '|' => Color::VIOLET,
        'g' => Color::GREEN,
        _ => Color::WHITE,
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

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player, origin: (i32, i32), scale: f32) {
    let mini = ((block_size as f32) * scale) as usize;
    let (ox, oy) = origin;

    // fondo del minimapa, es como un borde adicional a lo del maze
    // TODO: cambiar color
    framebuffer.set_current_color(Color::DARKBLUE);
    framebuffer.fill_rect(ox - 4, oy - 4, (maze[0].len() * mini + 8) as i32, (maze.len() * mini + 8) as i32);

    // Dibujar celdas, y hacerlo de cada uno de los colores
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * mini;
            let yo = row_index * mini;
            draw_cell_at(framebuffer, ox, oy, xo, yo, mini, cell);
        }
    }

    // jugador en el minimapa
    let px = (player.pos.x as usize / block_size) * mini;
    let py = (player.pos.y as usize / block_size) * mini;
    framebuffer.set_current_color(Color::YELLOW);
    framebuffer.fill_rect(ox + px as i32 - 2, oy + py as i32 - 2, 4, 4);
}

// Mundo 3D
fn render_world(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player) {
    // Tiramos un rayo por col del framebuffer
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0; // half height

    // Cielo
    // TODO: cambiar color
    framebuffer.set_current_color(Color::new(70, 80, 140, 255)); 
    framebuffer.fill_rect(0, 0, framebuffer.width as i32, hh as i32);

    // Piso
    // TODO: cambiar color
    framebuffer.set_current_color(Color::new(30, 30, 40, 255));
    framebuffer.fill_rect(0, hh as i32, framebuffer.width as i32, hh as i32);

    // La altura de la pared es proporcional a la distancia al plano de proyección
    // Si el fov es grande, la camara esta mas cerca y las paredes parecen mas altas
    // Esto no lo termino de entender
    // TODO: preguntar Erick
    let dist_plane = (framebuffer.width as f32 / 2.0) / (player.fov / 2.0).tan();

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let hit = cast_ray(framebuffer, maze, player, a, block_size, false);

        // TODO: chat me dijo que esto era para evitar el fisheye, no lo entiendo al 100, preguntar Erick
        let mut perp = hit.distance * (a - player.a).cos().abs();
        if perp < 0.0001 { perp = 0.0001; }

        // Escalabilidad de pixeles?
        let stake_height = ((block_size as f32) * dist_plane) / perp;

        // Posición vertical
        let stake_top = (hh - stake_height * 0.5) as i32;
        let stake_bottom = (hh + stake_height * 0.5) as i32;

        // Colores en base a pared tocada
        let base = match hit.impact { '+' => Color::BLUEVIOLET, '-' | '|' => Color::VIOLET, 'g' => Color::GREEN, _ => Color::RAYWHITE };
        let shade = ((1.0 / (1.0 + perp * 0.01)).clamp(0.2, 1.0) * 255.0) as u8;
        let wall_color = Color::new((base.r as u16 * shade as u16 / 255) as u8,
                                    (base.g as u16 * shade as u16 / 255) as u8,
                                    (base.b as u16 * shade as u16 / 255) as u8,
                                    255);
        framebuffer.set_current_color(wall_color);
        framebuffer.draw_vline(i, stake_top, stake_bottom);
    }
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
        .title("Raycaster 3D — baseline")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Color::new(50, 50, 100, 255));

    let maze = load_maze("maze.txt");

    // Posición inicial en 'p' (centro del bloque)
    let (start_i, start_j) = find_char(&maze, 'p').unwrap_or((1, 1));
    let start_x = (start_i * block_size + block_size / 2) as f32;
    let start_y = (start_j * block_size + block_size / 2) as f32;

    let mut player = Player {
        pos: Vector2::new(start_x, start_y),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    // Main render loop
    while !window.window_should_close() {
        // Resetea el lienzo
        framebuffer.clear();

        // actua acorde a inputs o colisiones
        process_events(&mut player, &window, &maze, block_size);

        // RENDER STEP
        // renderiza el mundo 3D
        render_world(&mut framebuffer, &maze, block_size, &player);
        // renderiza el minimapa
        render_minimap(&mut framebuffer, &maze, block_size, &player, (16, 16), 0.15);

        // Ya pasamos del Image al Texture2D, ahora se encarga de dibujar el Texture2D en la ventana
        framebuffer.swap_buffers(&mut window, &raylib_thread);

        // Tasa de refresco
        std::thread::sleep(Duration::from_millis(16));
    }
}