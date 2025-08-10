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

    pub fn draw_vline(&mut self, x: u32, y0: i32, y1: i32) {
        if x >= self.width { return; }
        let ys = y0.max(0) as u32;
        let ye = y1.min(self.height as i32).max(0) as u32;
        for y in ys..ye { self.set_pixel(x, y); }
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
        lost: bool,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let mut renderer = window.begin_drawing(raylib_thread);
            renderer.clear_background(Color::BLACK);
            renderer.draw_texture(&texture, 0, 0, Color::WHITE);

            // === HUD Overlay ===
            let sw = renderer.get_screen_width();
            let sh = renderer.get_screen_height();
            
            // FPS pill (arriba-derecha)
            renderer.draw_rectangle(sw - 110, 6, 100, 24, Color::new(0, 0, 0, 160));
            renderer.draw_fps(sw - 100, 10);

            // Coins pill
            let coins_label = format!("Keys: {}/{}", coins_collected, coins_total);
            let cw = renderer.measure_text(&coins_label, 20);
            let cpill_w = (cw + 20).max(122);
            let cpill_x = sw - cpill_w - 10;
            renderer.draw_rectangle(cpill_x, 34, cpill_w, 26, Color::new(235, 192, 121, 220)); // beige a juego
            renderer.draw_rectangle_lines(cpill_x, 34, cpill_w, 26, Color::new(24, 32, 56, 200)); // borde navy
            renderer.draw_text(&coins_label, cpill_x + 10, 38, 20, Color::GOLD);

            // Timer pill (arriba-izquierda)
            let mm = (time_left_secs / 60) as i32;
            let ss = (time_left_secs % 60) as i32;
            let time_label = format!("{:02}:{:02}", mm, ss);
            let tw = renderer.measure_text(&time_label, 22);
            renderer.draw_rectangle(10, 8, tw + 20, 28, Color::new(0, 0, 0, 160));
            renderer.draw_text(&time_label, 20, 12, 22, Color::RAYWHITE);

            // Overlay si perdiste
            if lost {
                renderer.draw_rectangle(0, 0, sw, sh, Color::new(0, 0, 0, 160));
                let msg = "¡PERDISTE!";
                let sub = "Te quedaste sin tiempo";
                let mw = renderer.measure_text(msg, 48);
                let sw2 = renderer.measure_text(sub, 24);
                renderer.draw_text(msg, (sw - mw)/2, sh/2 - 30, 48, Color::RED);
                renderer.draw_text(sub, (sw - sw2)/2, sh/2 + 22, 24, Color::RAYWHITE);
            }
            // === /HUD ===
        }
    }
}