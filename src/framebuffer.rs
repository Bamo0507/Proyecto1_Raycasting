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

            // ---------- FPS (top-right), aesthetic pill ----------
            let fps = renderer.get_fps();
            let fps_label = format!("FPS: {}", fps);
            let fps_font = 32; // bigger
            let fps_text_w = renderer.measure_text(&fps_label, fps_font);
            let pad_x = 14; let pad_y = 10;
            let box_w = fps_text_w + pad_x * 2;
            let box_h = fps_font + pad_y * 2;
            let x = sw - box_w - 12;
            let y = 12;
            // subtle shadow
            renderer.draw_rectangle(x + 3, y + 3, box_w, box_h, Color::new(0, 0, 0, 80));
            // pill base
            renderer.draw_rectangle(x, y, box_w, box_h, Color::new(10, 12, 20, 190));
            // accent border
            renderer.draw_rectangle_lines(x, y, box_w, box_h, Color::new(0, 218, 209, 210));
            renderer.draw_text(&fps_label, x + pad_x, y + pad_y, fps_font, Color::RAYWHITE);

            // ---------- Timer (bottom-left), bigger ----------
            let mm = (time_left_secs / 60) as i32;
            let ss = (time_left_secs % 60) as i32;
            let time_label = format!("{:02}:{:02}", mm, ss);
            let time_font = 34; // bigger
            let time_text_w = renderer.measure_text(&time_label, time_font);
            let t_pad_x = 16; let t_pad_y = 12;
            let t_box_w = time_text_w + t_pad_x * 2 + 40; // space for icon
            let t_box_h = time_font + t_pad_y * 2;
            let tx = 12;
            let ty = sh - t_box_h - 12;
            // shadow
            renderer.draw_rectangle(tx + 3, ty + 3, t_box_w, t_box_h, Color::new(0, 0, 0, 80));
            // base (beige to match theme)
            renderer.draw_rectangle(tx, ty, t_box_w, t_box_h, Color::new(235, 192, 121, 230));
            // border (navy)
            renderer.draw_rectangle_lines(tx, ty, t_box_w, t_box_h, Color::new(24, 32, 56, 220));
            // small clock icon (circle + top tick)
            let icx = tx + 18; let icy = ty + t_box_h/2;
            renderer.draw_circle(icx, icy, 12.0, Color::new(24, 32, 56, 255));
            renderer.draw_line(icx, icy - 8, icx, icy - 2, Color::new(235, 192, 121, 255));
            renderer.draw_line(icx, icy - 2, icx + 5, icy + 3, Color::new(235, 192, 121, 255));
            // text
            renderer.draw_text(&time_label, tx + 40 + t_pad_x, ty + t_pad_y, time_font, Color::new(24, 32, 56, 255));

            // ---------- Coins (bottom-right), aesthetic pill with coin icon ----------
            let coins_label = format!("Coins: {}/{}", coins_collected, coins_total);
            let coins_font = 32; // bigger
            let c_text_w = renderer.measure_text(&coins_label, coins_font);
            let c_pad_x = 16; let c_pad_y = 12;
            let c_box_w = c_text_w + c_pad_x * 2 + 42; // space for icon
            let c_box_h = coins_font + c_pad_y * 2;
            let cx = sw - c_box_w - 12; // bottom-right
            let cy = sh - c_box_h - 12;
            // shadow
            renderer.draw_rectangle(cx + 3, cy + 3, c_box_w, c_box_h, Color::new(0, 0, 0, 80));
            // base
            renderer.draw_rectangle(cx, cy, c_box_w, c_box_h, Color::new(10, 12, 20, 190));
            // border
            renderer.draw_rectangle_lines(cx, cy, c_box_w, c_box_h, Color::new(0, 218, 209, 210));
            // coin icon (double circle)
            let icx2 = cx + 22; let icy2 = cy + c_box_h/2;
            renderer.draw_circle(icx2, icy2, 12.0, Color::GOLD);
            renderer.draw_circle_lines(icx2, icy2, 12.0, Color::new(255, 219, 88, 255));
            renderer.draw_circle(icx2, icy2, 6.5, Color::new(255, 215, 120, 255));
            // text
            renderer.draw_text(&coins_label, cx + 40 + c_pad_x, cy + c_pad_y, coins_font, Color::RAYWHITE);
            // === /HUD ===
        }
    }
}