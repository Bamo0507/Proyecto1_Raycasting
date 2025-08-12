use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::{Maze, is_walkable};

pub struct Player {
    pub position: Vector2,  
    pub angle: f32,         
    pub field_of_view: f32, 
}

/// Procesa los eventos de entrada del teclado y mouse para controlar al jugador
/// Maneja la rotación con el mouse y teclas K/L, y movimiento con WASD, esto lo hace raylib al parecer jaja
pub fn process_events(player: &mut Player, raylib_handle: &RaylibHandle, maze: &Maze, block_size: usize) {
    const MOVEMENT_SPEED: f32 = 8.0;
    const MOUSE_SENSITIVITY: f32 = 0.006;
    const KEYBOARD_ROTATION_SPEED: f32 = std::f32::consts::PI / 60.0; // k/l para girar
    
    // Rotación con mouse
    let mouse_delta = raylib_handle.get_mouse_delta();
    player.angle += mouse_delta.x * MOUSE_SENSITIVITY;
    
    // Rotación con teclado
    if raylib_handle.is_key_down(KeyboardKey::KEY_L) { 
        player.angle += KEYBOARD_ROTATION_SPEED;
    }
    if raylib_handle.is_key_down(KeyboardKey::KEY_K) { 
        player.angle -= KEYBOARD_ROTATION_SPEED;
    }

    // Normaliza eangulo
    player.angle = player.angle.rem_euclid(2.0 * PI);
    if player.angle > PI { player.angle -= 2.0 * PI; }

    let mut forward_movement = 0.0;   // Movimiento hacia adelante/atrás
    let mut strafe_movement = 0.0;    // Movimiento lateral

    // Controles WASD
    if raylib_handle.is_key_down(KeyboardKey::KEY_W) { forward_movement += MOVEMENT_SPEED; }
    if raylib_handle.is_key_down(KeyboardKey::KEY_S) { forward_movement -= MOVEMENT_SPEED; }
    if raylib_handle.is_key_down(KeyboardKey::KEY_D) { strafe_movement -= MOVEMENT_SPEED; }
    if raylib_handle.is_key_down(KeyboardKey::KEY_A) { strafe_movement += MOVEMENT_SPEED; }

    if forward_movement != 0.0 || strafe_movement != 0.0 {
        // Vectores de dirección
        let forward_direction_x = player.angle.cos();
        let forward_direction_y = player.angle.sin();
        let right_direction_x = player.angle.sin();   
        let right_direction_y = -player.angle.cos();

        // Calcular el vector de movimiento combinado, por si hay movimiento en ambos ejes
        let mut movement_x = forward_movement * forward_direction_x + strafe_movement * right_direction_x;
        let mut movement_y = forward_movement * forward_direction_y + strafe_movement * right_direction_y;

        // Normalizar para que el movimiento diagonal no sea más rápido
        let movement_magnitude = (movement_x * movement_x + movement_y * movement_y).sqrt();
        if movement_magnitude > MOVEMENT_SPEED && movement_magnitude > 0.0 {
            let normalization_factor = MOVEMENT_SPEED / movement_magnitude;
            movement_x *= normalization_factor;
            movement_y *= normalization_factor;
        }

        // Detección de colisiones
        // Intento de movimiento en X
        let new_position_x = Vector2 { 
            x: player.position.x + movement_x, 
            y: player.position.y 
        };
        let grid_x = (new_position_x.x.max(0.0) as usize) / block_size;
        let grid_y = (new_position_x.y.max(0.0) as usize) / block_size;
        
        if grid_y < maze.len() && grid_x < maze[grid_y].len() && is_walkable(maze[grid_y][grid_x]) {
            player.position.x = new_position_x.x;
        }

        // Intento de movimiento en Y
        let new_position_y = Vector2 { 
            x: player.position.x, 
            y: player.position.y + movement_y 
        };
        let grid_x = (new_position_y.x.max(0.0) as usize) / block_size;
        let grid_y = (new_position_y.y.max(0.0) as usize) / block_size;
        
        if grid_y < maze.len() && grid_x < maze[grid_y].len() && is_walkable(maze[grid_y][grid_x]) {
            player.position.y = new_position_y.y;
        }
    }
}