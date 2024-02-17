pub enum ColorPalette {
    Classic,
    Inverted,
    Grayscale,
}

pub struct PaletteHandler {
    pub current_palette: ColorPalette,
}

impl PaletteHandler {
    pub fn new() -> Self {
        PaletteHandler {
            current_palette: ColorPalette::Classic, // Start with the classic palette
        }
    }

    pub fn cycle_palette(&mut self) {
        self.current_palette = match self.current_palette {
            ColorPalette::Classic => ColorPalette::Inverted,
            ColorPalette::Inverted => ColorPalette::Grayscale,
            ColorPalette::Grayscale => ColorPalette::Classic,
        };
    }

    pub fn calculate_color(&self, t: f64) -> (u8, u8, u8) {
        match self.current_palette {
            ColorPalette::Classic => self.classic_palette(t),
            ColorPalette::Inverted => self.inverted_palette(t),
            ColorPalette::Grayscale => self.grayscale_palette(t),
        }
    }

    pub fn classic_palette(&self, t: f64) -> (u8, u8, u8) {
        let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u8;
        let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u8;
        let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u8;
        (r, g, b)
    }

    pub fn inverted_palette(&self, t: f64) -> (u8, u8, u8) {
        let (r, g, b) = self.classic_palette(t);
        (255 - r, 255 - g, 255 - b)
    }

    pub fn grayscale_palette(&self, t: f64) -> (u8, u8, u8) {
        let intensity = (t * 255.0) as u8;
        (intensity, intensity, intensity)
    }
}
