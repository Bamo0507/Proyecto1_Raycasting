use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    // Se arranca el buffer inicial
    pub fn new(width: u32, height: u32) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
        }
    }

    // Recrea el Image con el background color
    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);
    }

    // Pinta un pixel con current color si esta dentro del buffer
    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    // Se define el color de fondo
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    // Se define un color, y en base a eso se pintan varios pixeles
    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    // Se utiliza para poder pintar un rectangulo, tomando en consideracion los
    // limites del buffer para que no se salga, sirve para el mapa 2D
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = (x + w).min(self.width as i32) as u32;
        let y1 = (y + h).min(self.height as i32) as u32;
        for yy in y0..y1 {
            for xx in x0..x1 {
                self.set_pixel(xx, yy);
            }
        }
    }

    //Dibuja una linea vertical, desde y0 hasta y1
    pub fn draw_vline(&mut self, x: u32, y0: i32, y1: i32) {
        if x >= self.width { return; }
        let ys = y0.max(0) as u32;
        let ye = y1.min(self.height as i32).max(0) as u32;
        for y in ys..ye { self.set_pixel(x, y); }
    }

    // Guarda el buffer en un archivo
    pub fn _render_to_file(&self, file_path: &str) {
        let _ = self.color_buffer.export_image(file_path);
    }

    // Se encarga de convertir el Image, que es lo del CPU a Texture2D, que es lo del GPU
    // y luego se encarga de dibujar el Texture2D en la ventana
    pub fn swap_buffers(&self, window: &mut RaylibHandle, raylib_thread: &RaylibThread) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let mut renderer = window.begin_drawing(raylib_thread);
            renderer.clear_background(Color::BLACK);
            renderer.draw_texture(&texture, 0, 0, Color::WHITE);
        }
    }
}