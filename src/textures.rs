use raylib::prelude::*;
use std::collections::HashMap;

pub struct TextureManager {
    images: HashMap<char, Image>,
}

impl TextureManager {
    /// Carga texturas para '-', '|', '+' componiendo wall+graffiti, y define fallbacks.
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, String> {
        let mut images: HashMap<char, Image> = HashMap::new();
        let mut textures: HashMap<char, Texture2D> = HashMap::new();

        // Mapa de caracteres a archivos (base wall + overlay graffiti)
        let defs: Vec<(char, &str, &str)> = vec![
            ('-', "assets/wall1.png", "assets/graffiti1.png"),
            ('|', "assets/wall2.png", "assets/graffiti2.png"),
            ('+', "assets/wall3.png", "assets/graffiti3.png"),
        ];

        // Cargar y componer
        for (ch, wall_path, graff_path) in defs {
            let mut wall = Image::load_image(wall_path).map_err(|e| format!("{}: {}", wall_path, e))?;
            let mut graf = Image::load_image(graff_path).map_err(|e| format!("{}: {}", graff_path, e))?;
            let composed = compose_overlay(&mut wall, &mut graf)?;
            let tex = rl
                .load_texture_from_image(thread, &composed)
                .map_err(|e| format!("load_texture_from_image: {}", e))?;
            images.insert(ch, composed);
            textures.insert(ch, tex);
        }

        // x default se usa - si hay un simbolo no definido
        if let Some(img) = images.get(&'-').cloned() {
            let tex = rl
                .load_texture_from_image(thread, &img)
                .map_err(|e| format!("fallback texture load: {}", e))?;
            images.insert('#', img);
            textures.insert('#', tex);
        }

        Ok(Self { images })
    }

    // Obtener el tamaño de la imagen
    pub fn get_image_size(&self, ch: char) -> (u32, u32) {
        if let Some(img) = self.images.get(&ch) {
            (img.width as u32, img.height as u32)
        } else if let Some(img) = self.images.get(&'#') {
            (img.width as u32, img.height as u32)
        } else {
            (1, 1)
        }
    }

    // Devuelve el color en (tx, ty) del Image mapeado al simbolo
    pub fn get_pixel_color_mut(&mut self, ch: char, tx: u32, ty: u32) -> Color {
        let key = if self.images.contains_key(&ch) { ch } else { '#' };
        if let Some(img) = self.images.get_mut(&key) {
            let w = img.width.max(1) as u32;
            let h = img.height.max(1) as u32;
            let x = tx.min(w - 1) as i32;
            let y = ty.min(h - 1) as i32;
            return img.get_color(x, y);
        }
        Color::WHITE
    }
}

// Metodo para poder tener wall de base y el graffiti encima (ambas son 256x256)
fn compose_overlay(base: &mut Image, overlay: &mut Image) -> Result<Image, String> {
    let bw = base.width as i32; let bh = base.height as i32;
    if overlay.width != base.width || overlay.height != base.height {
        return Err("wall.png y graffiti.png deben tener el mismo tamaño".into());
    }
    let mut out = Image::gen_image_color(bw, bh, Color::BLACK);
    for y in 0..bh {
        for x in 0..bw {
            let bc = base.get_color(x, y);
            let oc = overlay.get_color(x, y);
            let a  = oc.a as u16; let ai = 255u16 - a;
            let r = ((oc.r as u16 * a + bc.r as u16 * ai) / 255) as u8;
            let g = ((oc.g as u16 * a + bc.g as u16 * ai) / 255) as u8;
            let b = ((oc.b as u16 * a + bc.b as u16 * ai) / 255) as u8;
            out.draw_pixel(x, y, Color::new(r, g, b, 255));
        }
    }
    Ok(out)
}