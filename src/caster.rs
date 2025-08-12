use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, is_walkable};
use crate::player::Player;

/// Representa el resultado de la intersección de un rayo con el entorno.
/// Da toda la información sobre el impacto, como distancia de choque,
/// el tipo de pared impactada y la coordenada de textura para el mapeo.
pub struct Intersect {
    pub hit_distance: f32,
    pub wall_type: char,
    pub texture_coord: f32,
}

/// Lanza un rayo desde la posición del jugador en un ángulo específico
/// y devuelve información sobre la primera intersección con un objeto
pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    angle_rad: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    // Longitud actual del rayo
    let mut ray_length: f32 = 0.0;
    // step para cada cuantos px tiramos rayo
    let ray_step_size: f32 = 1.0; 

    framebuffer.set_current_color(Color::WHITESMOKE);

    let block_size_f32 = block_size as f32;
    let mut prev_world_x = player.position.x;
    let mut prev_world_y = player.position.y;

    loop {
        // Se determina la posicion del rayo en el mundo
        let world_x = player.position.x + ray_length * angle_rad.cos();
        let world_y = player.position.y + ray_length * angle_rad.sin();

        // Verificar si el rayo ha salido de los límites del mundo
        if world_x < 0.0 || world_y < 0.0 {
            return Intersect { 
                hit_distance: ray_length, 
                wall_type: '#', 
                texture_coord: 0.0 
            };
        }

        let pixel_x = world_x as usize;
        let pixel_y = world_y as usize;

        // Obtenemos la celda actual en la cuadrícula del laberinto
        let grid_x = pixel_x / block_size;
        let grid_y = pixel_y / block_size;

        // Verificar si estamos fuera de los límites del laberinto
        if grid_y >= maze.len() || grid_x >= maze[grid_y].len() {
            return Intersect { 
                hit_distance: ray_length, 
                wall_type: '#', 
                texture_coord: 0.0 
            };
        }

        let cell_type = maze[grid_y][grid_x];
        if !is_walkable(cell_type) {
            // Obtenemos la celda anterior (desde donde veníamos)
            let prev_grid_x = (prev_world_x as usize) / block_size;
            let prev_grid_y = (prev_world_y as usize) / block_size;

            let cell_origin_x = (grid_x * block_size) as f32;
            let cell_origin_y = (grid_y * block_size) as f32;
            let local_x = world_x - cell_origin_x;
            let local_y = world_y - cell_origin_y;

            // Se determina si estamos en un borde
            let texture_coord = if grid_x != prev_grid_x && grid_y == prev_grid_y {
                (local_y / block_size_f32).fract()
            } else if grid_y != prev_grid_y && grid_x == prev_grid_x {
                (local_x / block_size_f32).fract()
            } else {
                let edge_x = local_x.min(block_size_f32 - local_x);
                let edge_y = local_y.min(block_size_f32 - local_y);
                if edge_x < edge_y { 
                    (local_y / block_size_f32).fract() 
                } else { 
                    (local_x / block_size_f32).fract() 
                }
            };

            return Intersect { 
                hit_distance: ray_length, 
                wall_type: cell_type, 
                texture_coord 
            };
        }

        // Dibujar el rayo si está habilitado
        if draw_line {
            framebuffer.set_pixel(pixel_x as u32, pixel_y as u32);
        }
        
        // Actualizar posición anterior y avanzar el rayo
        prev_world_x = world_x;
        prev_world_y = world_y;
        ray_length += ray_step_size;
    }
}