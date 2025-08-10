use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::{Maze, is_walkable};

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32,
}

// Manejo de logica del jugador
// Inputs, movimientos y salidas
// Raylib es lo que permite hacer el manejo de lectura del teclado
pub fn process_events(player: &mut Player, rl: &RaylibHandle, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 10.0; // píxeles por movimiento
    const ROTATION_SPEED: f32 = PI / 30.0; // grados por movimiento

    // Girar
    if rl.is_key_down(KeyboardKey::KEY_RIGHT)  { player.a += ROTATION_SPEED; }
    if rl.is_key_down(KeyboardKey::KEY_LEFT) { player.a -= ROTATION_SPEED; }

    let mut next = player.pos;
    // Avanzar
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        next.x += MOVE_SPEED * player.a.cos();
        next.y += MOVE_SPEED * player.a.sin();
    }
    // Retroceder
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        next.x -= MOVE_SPEED * player.a.cos();
        next.y -= MOVE_SPEED * player.a.sin();
    }

    // Colisiones contra paredes
    let i = (next.x.max(0.0) as usize) / block_size;
    let j = (next.y.max(0.0) as usize) / block_size;
    // Si estamos dentro de los limites y estamos en un 'walkable zone' desplazamos
    if j < maze.len() && i < maze[j].len() && is_walkable(maze[j][i]) {
        player.pos = next;
    }
}