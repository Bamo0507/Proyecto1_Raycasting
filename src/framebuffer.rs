use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
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

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

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

    pub fn _render_to_file(&self, file_path: &str) {
        let _ = self.color_buffer.export_image(file_path);
    }

    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        coins_collected: usize,
        coins_total: usize,
        time_left_secs: u32,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let mut renderer = window.begin_drawing(raylib_thread);
            renderer.clear_background(Color::BLACK);
            renderer.draw_texture(&texture, 0, 0, Color::WHITE);

            // Tamanios para el HUD
            let screen_width = renderer.get_screen_width();
            let screen_height = renderer.get_screen_height();

            // Seteo para FPS
            let fps = renderer.get_fps();
            let fps_label = format!("FPS: {}", fps);
            let fps_font_size = 32;
            let fps_text_width = renderer.measure_text(&fps_label, fps_font_size);
            let fps_padding_x = 14;
            let fps_padding_y = 10;
            let fps_box_width = fps_text_width + fps_padding_x * 2;
            let fps_box_height = fps_font_size + fps_padding_y * 2;
            let fps_box_x = screen_width - fps_box_width - 12;
            let fps_box_y = 12;
            
            // Dibujar rectangulos de FPS 
            // Cada uno con colores diferentes x estetica
            renderer.draw_rectangle(
                fps_box_x + 3, 
                fps_box_y + 3, 
                fps_box_width, 
                fps_box_height, 
                Color::new(0, 0, 0, 80)
            );
            renderer.draw_rectangle(
                fps_box_x, 
                fps_box_y, 
                fps_box_width, 
                fps_box_height, 
                Color::new(10, 12, 20, 190)
            );
            renderer.draw_rectangle_lines(
                fps_box_x, 
                fps_box_y, 
                fps_box_width, 
                fps_box_height, 
                Color::new(0, 218, 209, 210)
            );
            renderer.draw_text(
                &fps_label, 
                fps_box_x + fps_padding_x, 
                fps_box_y + fps_padding_y, 
                fps_font_size, 
                Color::RAYWHITE
            );

            // Seteo para Timer
            let minutes_left = (time_left_secs / 60) as i32;
            let seconds_left = (time_left_secs % 60) as i32;
            let time_label = format!("{:02}:{:02}", minutes_left, seconds_left);
            let time_font_size = 34;
            let time_text_width = renderer.measure_text(&time_label, time_font_size);
            let time_padding_x = 16;
            let time_padding_y = 12;
            let time_box_width = time_text_width + time_padding_x * 2 + 40; // Space for icon
            let time_box_height = time_font_size + time_padding_y * 2;
            let time_box_x = 12;
            let time_box_y = screen_height - time_box_height - 12;
            
            // Dibujar rectangulos de Timer
            renderer.draw_rectangle(
                time_box_x + 3, 
                time_box_y + 3, 
                time_box_width, 
                time_box_height, 
                Color::new(0, 0, 0, 80)
            );
            renderer.draw_rectangle(
                time_box_x, 
                time_box_y, 
                time_box_width, 
                time_box_height, 
                Color::new(235, 192, 121, 230)
            );
            renderer.draw_rectangle_lines(
                time_box_x, 
                time_box_y, 
                time_box_width, 
                time_box_height, 
                Color::new(24, 32, 56, 220)
            );
            
            // Dibujar icono de reloj
            let clock_center_x = time_box_x + 18;
            let clock_center_y = time_box_y + time_box_height / 2;
            renderer.draw_circle(
                clock_center_x, 
                clock_center_y, 
                12.0, 
                Color::new(24, 32, 56, 255)
            );
            // Dibujar brazos del reloj
            let clock_hand_color = Color::new(235, 192, 121, 255);
            renderer.draw_line(
                clock_center_x, 
                clock_center_y - 8, 
                clock_center_x, 
                clock_center_y - 2, 
                clock_hand_color
            );
            renderer.draw_line(
                clock_center_x, 
                clock_center_y - 2, 
                clock_center_x + 5, 
                clock_center_y + 3, 
                clock_hand_color
            );
            
            // Dibujar texto del timer
            renderer.draw_text(
                &time_label, 
                time_box_x + 40 + time_padding_x, 
                time_box_y + time_padding_y, 
                time_font_size, 
                Color::new(24, 32, 56, 255)
            );

            // Seteo para Coin Counter
            let coins_label = format!("Coins: {}/{}", coins_collected, coins_total);
            let coin_font_size = 32;
            let coin_text_width = renderer.measure_text(&coins_label, coin_font_size);
            let coin_padding_x = 16;
            let coin_padding_y = 12;
            let coin_box_width = coin_text_width + coin_padding_x * 2 + 42; // Space for icon
            let coin_box_height = coin_font_size + coin_padding_y * 2;
            let coin_box_x = screen_width - coin_box_width - 12;
            let coin_box_y = screen_height - coin_box_height - 12;
            
            // Dibujar rectangulos de Coin Counter
            renderer.draw_rectangle(
                coin_box_x + 3, 
                coin_box_y + 3, 
                coin_box_width, 
                coin_box_height, 
                Color::new(0, 0, 0, 80)
            );
            renderer.draw_rectangle(
                coin_box_x, 
                coin_box_y, 
                coin_box_width, 
                coin_box_height, 
                Color::new(10, 12, 20, 190)
            );
            renderer.draw_rectangle_lines(
                coin_box_x, 
                coin_box_y, 
                coin_box_width, 
                coin_box_height, 
                Color::new(0, 218, 209, 210)
            );
            
            // Dibujar icono de moneda
            let coin_center_x = coin_box_x + 22;
            let coin_center_y = coin_box_y + coin_box_height / 2;
            renderer.draw_circle(coin_center_x, coin_center_y, 12.0, Color::GOLD);
            renderer.draw_circle_lines(
                coin_center_x, 
                coin_center_y, 
                12.0, 
                Color::new(255, 219, 88, 255)
            );
            renderer.draw_circle(
                coin_center_x, 
                coin_center_y, 
                6.5, 
                Color::new(255, 215, 120, 255)
            );
            
            // Dibujar texto de Coin Counter
            renderer.draw_text(
                &coins_label, 
                coin_box_x + 40 + coin_padding_x, 
                coin_box_y + coin_padding_y, 
                coin_font_size, 
                Color::RAYWHITE
            );
        }
    }
}