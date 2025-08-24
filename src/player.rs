use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::{Maze, is_walkable};

pub struct Player {
    pub position: Vector2,  
    pub angle: f32,         
    pub field_of_view: f32, 
    pub collision_radius: f32,
}

impl Player {
    pub fn new(position: Vector2, angle: f32, field_of_view: f32) -> Self {
        Self {
            position,
            angle,
            field_of_view,
            collision_radius: 20.0,
        }
    }
}

// Función auxiliar para verificar colisiones con el radio
fn check_collision(position: Vector2, radius: f32, maze: &Maze, block_size: usize) -> bool {
    // Verificar en múltiples puntos alrededor del jugador para una detección más precisa
    let points = [
        position,  // Centro
        Vector2 { x: position.x + radius, y: position.y },
        Vector2 { x: position.x - radius, y: position.y },
        Vector2 { x: position.x, y: position.y + radius },
        Vector2 { x: position.x, y: position.y - radius },
    ];

    for point in &points {
        let grid_x = (point.x.max(0.0) as usize) / block_size;
        let grid_y = (point.y.max(0.0) as usize) / block_size;
        
        // Si alguna parte del radio del jugador toca una pared, hay colisión
        if grid_y >= maze.len() || grid_x >= maze[0].len() || !is_walkable(maze[grid_y][grid_x]) {
            return true;
        }
    }
    
    false
}

/// Procesa los eventos de entrada del teclado y mouse para controlar al jugador
pub fn process_events(player: &mut Player, raylib_handle: &RaylibHandle, maze: &Maze, block_size: usize) {
    const MOVEMENT_SPEED: f32 = 10.0;
    const MOUSE_SENSITIVITY: f32 = 0.008;
    const KEYBOARD_ROTATION_SPEED: f32 = std::f32::consts::PI / 40.0;
    
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

    // Normaliza el ángulo
    player.angle = player.angle.rem_euclid(2.0 * PI);
    if player.angle > PI { player.angle -= 2.0 * PI; }

    let mut forward_movement = 0.0;
    let mut strafe_movement = 0.0;

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

        // Calcular el vector de movimiento combinado
        let mut movement_x = forward_movement * forward_direction_x + strafe_movement * right_direction_x;
        let mut movement_y = forward_movement * forward_direction_y + strafe_movement * right_direction_y;

        // Normalizar movimiento diagonal
        let movement_magnitude = (movement_x * movement_x + movement_y * movement_y).sqrt();
        if movement_magnitude > MOVEMENT_SPEED && movement_magnitude > 0.0 {
            let normalization_factor = MOVEMENT_SPEED / movement_magnitude;
            movement_x *= normalization_factor;
            movement_y *= normalization_factor;
        }

        // Verificar colisiones en el nuevo movimiento
        let new_position = Vector2 {
            x: player.position.x + movement_x,
            y: player.position.y + movement_y
        };

        // Solo actualizar la posición si no hay colisión
        if !check_collision(new_position, player.collision_radius, maze, block_size) {
            player.position = new_position;
        } else {
            // Si hay colisión, intentar mover solo en X o solo en Y
            let try_x = Vector2 {
                x: player.position.x + movement_x,
                y: player.position.y
            };
            
            let try_y = Vector2 {
                x: player.position.x,
                y: player.position.y + movement_y
            };
            
            if !check_collision(try_x, player.collision_radius, maze, block_size) {
                player.position.x = try_x.x;
            }
            if !check_collision(try_y, player.collision_radius, maze, block_size) {
                player.position.y = try_y.y;
            }
        }
    }
}