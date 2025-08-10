use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, is_walkable};
use crate::player::Player;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub hit_x: f32,
    pub hit_y: f32,
    pub tex_frac: f32, // [0,1) fracción horizontal dentro de la textura
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    // Paso del rayo en píxeles; más pequeño = más preciso
    let mut d: f32 = 0.0;
    let step: f32 = 1.0;

    framebuffer.set_current_color(Color::WHITESMOKE);

    let bs = block_size as f32;
    let mut prev_wx = player.pos.x;
    let mut prev_wy = player.pos.y;

    loop {
        let wx = player.pos.x + d * a.cos();
        let wy = player.pos.y + d * a.sin();

        if wx < 0.0 || wy < 0.0 {
            return Intersect { distance: d, impact: '#', hit_x: wx, hit_y: wy, tex_frac: 0.0 };
        }
        let x = wx as usize;
        let y = wy as usize;

        // Índices de celda actuales
        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            // Fuera del laberinto: tratamos como pared lejana
            return Intersect { distance: d, impact: '#', hit_x: wx, hit_y: wy, tex_frac: 0.0 };
        }

        let cell = maze[j][i];
        if !is_walkable(cell) {
            // Celda anterior (desde donde veníamos en el paso anterior)
            let pi = (prev_wx as usize) / block_size;
            let pj = (prev_wy as usize) / block_size;

            // Coordenadas locales dentro de la celda impactada [0, bs)
            let cell_origin_x = (i * block_size) as f32;
            let cell_origin_y = (j * block_size) as f32;
            let lx = wx - cell_origin_x;
            let ly = wy - cell_origin_y;

            // Determinar si cruzamos un borde vertical u horizontal
            // i != pi  => borde vertical (usamos ly)
            // j != pj  => borde horizontal (usamos lx)
            let tex_frac = if i != pi && j == pj {
                (ly / bs).fract()
            } else if j != pj && i == pi {
                (lx / bs).fract()
            } else {
                // esquina: decide por el borde más cercano
                let edge_x = lx.min(bs - lx);
                let edge_y = ly.min(bs - ly);
                if edge_x < edge_y { (ly / bs).fract() } else { (lx / bs).fract() }
            };

            return Intersect { distance: d, impact: cell, hit_x: wx, hit_y: wy, tex_frac };
        }

        if draw_line {
            framebuffer.set_pixel(x as u32, y as u32);
        }
        prev_wx = wx;
        prev_wy = wy;
        d += step;
    }
}