pub type Rgb = (u8, u8, u8);
pub type Vec3 = (f64, f64, f64);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorPalette {
    Classic,
    Inverted,
    Viridis,
    Plasma,
    Magma,
    Inferno,
    Grayscale,
    NewtonRapshon,
    Custom1,
    Custom2,
}

impl ColorPalette {
    // NOTE: this should be updated whenever a field is added
    // TODO: look if there is a way to add a Count entry in the enum to track it directly
    fn variant_count() -> u8 {
        10
    }

    fn from_index(index: u8) -> Option<ColorPalette> {
        match index {
            0 => Some(ColorPalette::Classic),
            1 => Some(ColorPalette::Inverted),
            2 => Some(ColorPalette::Viridis),
            3 => Some(ColorPalette::Magma),
            4 => Some(ColorPalette::Plasma),
            5 => Some(ColorPalette::Inferno),
            6 => Some(ColorPalette::Grayscale),
            7 => Some(ColorPalette::NewtonRapshon),
            8 => Some(ColorPalette::Custom1),
            9 => Some(ColorPalette::Custom2),
            _ => None,
        }
    }
}

pub struct PaletteHandler {
    pub current_palette: ColorPalette,
}

impl PaletteHandler {
    pub fn new() -> Self {
        PaletteHandler {
            current_palette: ColorPalette::Classic,
        }
    }

    pub fn cycle_palette_forward(&mut self) {
        let palette_count = ColorPalette::variant_count();
        let current_index = self.current_palette as u8;
        let next_index = (current_index + 1) % palette_count;
        self.current_palette = ColorPalette::from_index(next_index).unwrap();
    }

    pub fn cycle_palette_backward(&mut self) {
        let palette_count = ColorPalette::variant_count();
        let current_index = self.current_palette as u8;
        let next_index = (current_index + palette_count - 1) % palette_count;
        self.current_palette = ColorPalette::from_index(next_index).unwrap();
    }

    pub fn calculate_color(&self, t: f64) -> Rgb {
        match self.current_palette {
            ColorPalette::Classic => self.classic_palette(t),
            ColorPalette::Inverted => self.inverted_palette(t),
            ColorPalette::Grayscale => self.grayscale_palette(t),
            ColorPalette::Viridis => self.viridis_palette(t),
            ColorPalette::Plasma => self.plasma_palette(t),
            ColorPalette::Magma => self.magma_palette(t),
            ColorPalette::Inferno => self.inferno_palette(t),
            ColorPalette::NewtonRapshon => self.newton_raphson_palette(t),
            _ => self.custom_palette(t),
        }
    }

    pub fn classic_palette(&self, t: f64) -> Rgb {
        let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u8;
        let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u8;
        let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u8;
        (r, g, b)
    }

    pub fn inverted_palette(&self, t: f64) -> Rgb {
        let (r, g, b) = self.classic_palette(t);
        (255 - r, 255 - g, 255 - b)
    }

    pub fn grayscale_palette(&self, t: f64) -> Rgb {
        let intensity = (t * 255.0) as u8;
        (intensity, intensity, intensity)
    }

    pub fn calculate_custom_palette(&self, t: f64, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Rgb {
        let calc = |idx: usize| -> f64 {
            match idx {
                0 => a.0 + b.0 * (6.283185 * (c.0 * t + d.0)).cos(),
                1 => a.1 + b.1 * (6.283185 * (c.1 * t + d.1)).cos(),
                2 => a.2 + b.2 * (6.283185 * (c.2 * t + d.2)).cos(),
                _ => 0.0,
            }
        };

        let r = calc(0) * 255.0;
        let g = calc(1) * 255.0;
        let b = calc(2) * 255.0;

        (
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
        )
    }

    pub fn custom_palette(&self, t: f64) -> Rgb {
        fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
            (x, y, z)
        }

        match self.current_palette {
            ColorPalette::Custom1 => self.calculate_custom_palette(
                t,
                vec3(0.5, 0.5, 0.5),
                vec3(0.5, 0.5, 0.5),
                vec3(1.0, 1.0, 1.0),
                vec3(0.00, 0.33, 0.67),
            ),
            ColorPalette::Custom2 => self.calculate_custom_palette(
                t,
                vec3(0.5, 0.5, 0.5),
                vec3(0.5, 0.5, 0.5),
                vec3(1.0, 1.0, 1.0),
                vec3(0.00, 0.10, 0.20),
            ),
            _ => (0, 0, 0),
        }
    }

    pub fn viridis_palette(&self, t: f64) -> Rgb {
        fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
            (x, y, z)
        }

        let c0 = vec3(0.2777273272234177, 0.005407344544966578, 0.3340998053353061);
        let c1 = vec3(0.1050930431085774, 1.404613529898575, 1.384590162594685);
        let c2 = vec3(-0.3308618287255563, 0.214847559468213, 0.09509516302823659);
        let c3 = vec3(-4.634230498983486, -5.799100973351585, -19.33244095627987);
        let c4 = vec3(6.228269936347081, 14.17993336680509, 56.69055260068105);
        let c5 = vec3(4.776384997670288, -13.74514537774601, -65.35303263337234);
        let c6 = vec3(-5.435455855934631, 4.645852612178535, 26.3124352495832);

        let color =
            c0.0 + t * (c1.0 + t * (c2.0 + t * (c3.0 + t * (c4.0 + t * (c5.0 + t * c6.0)))));
        let color_g =
            c0.1 + t * (c1.1 + t * (c2.1 + t * (c3.1 + t * (c4.1 + t * (c5.1 + t * c6.1)))));
        let color_b =
            c0.2 + t * (c1.2 + t * (c2.2 + t * (c3.2 + t * (c4.2 + t * (c5.2 + t * c6.2)))));

        let color = (
            (color * 255.0).clamp(0.0, 255.0) as u8,
            (color_g * 255.0).clamp(0.0, 255.0) as u8,
            (color_b * 255.0).clamp(0.0, 255.0) as u8,
        );

        color
    }

    pub fn plasma_palette(&self, t: f64) -> Rgb {
        self.calculate_color_from_coefficients(
            (
                0.05873234392399702,
                2.176514634195958,
                -2.689460476458034,
                6.130348345893603,
                -11.10743619062271,
                10.02306557647065,
                -3.658713842777788,
            ),
            (
                0.02333670892565664,
                0.2383834171260182,
                -7.455851135738909,
                42.3461881477227,
                -82.66631109428045,
                71.41361770095349,
                -22.93153465461149,
            ),
            (
                0.5433401826748754,
                0.7539604599784036,
                3.110799939717086,
                -28.51885465332158,
                60.13984767418263,
                -54.07218655560067,
                18.19190778539828,
            ),
            t,
        )
    }

    pub fn magma_palette(&self, t: f64) -> Rgb {
        self.calculate_color_from_coefficients(
            (
                -0.002136485053939582,
                0.2516605407371642,
                8.353717279216625,
                -27.66873308576866,
                52.17613981234068,
                -50.76852536473588,
                18.65570506591883,
            ),
            (
                -0.000749655052795221,
                0.6775232436837668,
                -3.577719514958484,
                14.26473078096533,
                -27.94360607168351,
                29.04658282127291,
                -11.48977351997711,
            ),
            (
                -0.005386127855323933,
                2.494026599312351,
                0.3144679030132573,
                -13.64921318813922,
                12.94416944238394,
                4.23415299384598,
                -5.601961508734096,
            ),
            t,
        )
    }

    pub fn inferno_palette(&self, t: f64) -> Rgb {
        self.calculate_color_from_coefficients(
            (
                0.0002189403691192265,
                0.1065134194856116,
                11.60249308247187,
                -41.70399613139459,
                77.162935699427,
                -71.31942824499214,
                25.13112622477341,
            ),
            (
                0.001651004631001012,
                0.5639564367884091,
                -3.972853965665698,
                17.43639888205313,
                -33.40235894210092,
                32.62606426397723,
                -12.24266895238567,
            ),
            (
                -0.01948089843709184,
                3.932712388889277,
                -15.9423941062914,
                44.35414519872813,
                -81.80730925738993,
                73.20951985803202,
                -23.07032500287172,
            ),
            t,
        )
    }

    // BUG: Only the green channel is visible
    pub fn newton_raphson_palette(&self, t: f64) -> Rgb {
        let t = t.clamp(0.0, 1.0);

        let green_start = 0.0;
        let blue_start = 0.33;
        let red_start = 0.66;
        let (r, g, b) = match t {
            t if t < blue_start => {
                let progress = (t - green_start) / (blue_start - green_start);
                let green = (1.0 - progress) * 255.0;
                let blue = progress * 255.0;
                (0.0, green, blue)
            }
            t if t < red_start => {
                let progress = (t - blue_start) / (red_start - blue_start);
                let blue = (1.0 - progress) * 255.0;
                let red = progress * 255.0;
                (red, 0.0, blue)
            }
            _ => {
                let progress = (t - red_start) / (1.0 - red_start);
                let red = (1.0 - progress) * 255.0;
                let green = progress * 255.0;
                (red, green, 0.0)
            }
        };

        (r as u8, g as u8, b as u8)
    }

    fn calculate_color_from_coefficients(
        &self,
        r_coeffs: (f64, f64, f64, f64, f64, f64, f64),
        g_coeffs: (f64, f64, f64, f64, f64, f64, f64),
        b_coeffs: (f64, f64, f64, f64, f64, f64, f64),
        t: f64,
    ) -> Rgb {
        let calculate_channel = |coeffs: (f64, f64, f64, f64, f64, f64, f64)| {
            let (c0, c1, c2, c3, c4, c5, c6) = coeffs;
            let value = c0 + t * (c1 + t * (c2 + t * (c3 + t * (c4 + t * (c5 + t * c6)))));
            (value * 255.0).clamp(0.0, 255.0) as u8
        };

        (
            calculate_channel(r_coeffs),
            calculate_channel(g_coeffs),
            calculate_channel(b_coeffs),
        )
    }
}
