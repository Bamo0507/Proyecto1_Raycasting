use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, is_walkable};
use crate::player::Player;

// Interseccion entre el rayo y la pared
// Se tiene el caracter contra el que se choco y la distancia a la que fue
pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32, // Angulo del rayo
    block_size: usize, // Tamanio de pixeles
    draw_line: bool, // Dibujar la linea del rayo
) -> Intersect {
    // Paso del rayo en píxeles
    let mut d: f32 = 0.0;
    let step: f32 = 2.0;

    // Color de linea
    framebuffer.set_current_color(Color::WHITESMOKE);

    loop {
        // Avanzar d pixeles en direccion del angulo
        let x = (player.pos.x + d * a.cos()) as i32;
        let y = (player.pos.y + d * a.sin()) as i32;

        // Manejo de limites, devuelve el intersecto si topa con algo
        if x < 0 || y < 0 { return Intersect { distance: d, impact: '#' }; }

        // Se convierte a usize para poder acceder a la matriz
        let x = x as usize;
        let y = y as usize;
        // Se divide por el tamanio de la celda para obtener la posicion en la matriz
        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return Intersect { distance: d, impact: '#'};
        }

        // Se obtiene la celda y se valida si se puede caminar
        let cell = maze[j][i];
        if !is_walkable(cell) {
            // Si no se puede caminar, se retorna el intersecto
            return Intersect { distance: d, impact: cell };
        }

        // validar para dibujar rayo en el mapa
        if draw_line { framebuffer.set_pixel(x as u32, y as u32); }

        // Avanzar
        d += step;
    }
}