use std::fs::File;
use std::io::{BufRead, BufReader};

// Representamos el laberinto como una matriz de caracteres
pub type Maze = Vec<Vec<char>>;

// Construccion del laberinto
pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).expect("No se pudo abrir maze.txt");
    let reader = BufReader::new(file);
    reader.lines().map(|l| l.unwrap().chars().collect()).collect()
}

// Busqueda de un caracter en el laberinto, se usa para poner al player en p
pub fn find_char(maze: &Maze, ch: char) -> Option<(usize, usize)> {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == ch { return Some((i, j)); }
        }
    }
    None
}

// Validacion de que se pueda caminar
pub fn is_walkable(cell: char) -> bool {
    matches!(cell, ' ' | 'p' | 'g')
}