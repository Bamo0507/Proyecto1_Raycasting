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
    const MOVE_SPEED: f32 = 8.0;           // velocidad de avance/retroceso y strafe
    const MOUSE_SENS: f32 = 0.005;         // sensibilidad de rotación por mouse (solo horizontal)

    // --- Rotación SOLO con el mouse ---
    let md = rl.get_mouse_delta();
    player.a += md.x * MOUSE_SENS; // mover mouse a la derecha => gira a la derecha

    // Normaliza el ángulo a [-PI, PI] para evitar overflow con el tiempo
    if player.a >  PI { player.a -= 2.0 * PI; }
    if player.a < -PI { player.a += 2.0 * PI; }

    // --- Movimiento: W (adelante), S (atrás), A (izquierda/strafe), D (derecha/strafe) ---
    let mut move_step = 0.0;   // adelante/atrás a lo largo de la vista
    let mut strafe_step = 0.0; // izquierda/derecha perpendicular a la vista

    if rl.is_key_down(KeyboardKey::KEY_W) { move_step   += MOVE_SPEED; }
    if rl.is_key_down(KeyboardKey::KEY_S) { move_step   -= MOVE_SPEED; }
    if rl.is_key_down(KeyboardKey::KEY_D) { strafe_step -= MOVE_SPEED; } // derecha
    if rl.is_key_down(KeyboardKey::KEY_A) { strafe_step += MOVE_SPEED; } // izquierda

    if move_step != 0.0 || strafe_step != 0.0 {
        // Vectores base
        let forward_x = player.a.cos();
        let forward_y = player.a.sin();
        let right_x   =  player.a.sin();   // derecha = +90° de forward
        let right_y   = -player.a.cos();

        // Componer desplazamiento deseado
        let mut step_x = move_step * forward_x + strafe_step * right_x;
        let mut step_y = move_step * forward_y + strafe_step * right_y;

        // Normalizar para que diagonal no sea más rápida que MOVE_SPEED
        let mag = (step_x * step_x + step_y * step_y).sqrt();
        if mag > MOVE_SPEED && mag > 0.0 {
            let k = MOVE_SPEED / mag;
            step_x *= k;
            step_y *= k;
        }

        // Colisiones por ejes (permite deslizarse por paredes)
        // 1) intento en X
        let try_x = Vector2 { x: player.pos.x + step_x, y: player.pos.y };
        let ix = (try_x.x.max(0.0) as usize) / block_size;
        let jx = (try_x.y.max(0.0) as usize) / block_size;
        if jx < maze.len() && ix < maze[jx].len() && is_walkable(maze[jx][ix]) {
            player.pos.x = try_x.x;
        }

        // 2) intento en Y
        let try_y = Vector2 { x: player.pos.x, y: player.pos.y + step_y };
        let iy = (try_y.x.max(0.0) as usize) / block_size;
        let jy = (try_y.y.max(0.0) as usize) / block_size;
        if jy < maze.len() && iy < maze[jy].len() && is_walkable(maze[jy][iy]) {
            player.pos.y = try_y.y;
        }
    }
}