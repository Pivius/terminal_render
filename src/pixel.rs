use crossterm::style::Color;

/// Vector2
#[derive(Clone, Copy, Default)]
pub struct Vector2 {
    x: u32,
    y: u32,
}

#[macro_export]
macro_rules! vector2 {
    ($x:expr, $y:expr) => {
        Vector2::new($x, $y)
    };
}

impl Vector2 {
    pub fn new(x: u32, y:u32) -> Self {
        Self {x, y}
    }

    pub fn get_x(&self) -> u32 {
        self.x
    }

    pub fn get_y(&self) -> u32 {
        self.y
    }

    pub fn set_x(&mut self, x: u32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: u32) {
        self.y = y;
    }
}

/// Pixel
#[derive(Clone, Copy, Default)]
pub struct PxData {
    r: u8,
    g: u8,
    b: u8,
    position: Vector2,
}

#[macro_export]
macro_rules! pixel {
    ($r:expr, $g:expr, $b:expr, $x:expr, $y:expr) => {
        PxData::new(Color::Rgb{r: $r, g: $g, b: $b}, Vector2::new($x, $y))
    };
}

impl PxData {
    pub fn new(color: Color, position: Vector2) -> Self {
        match color {
            Color::Rgb{r, g, b} => {
                Self {r, g, b, position}
            }
            _ => {
                Self {r: 0, g: 0, b: 0, position}
            }
        }
    }

    pub fn get_r(&self) -> u8 {
        self.r
    }

    pub fn get_g(&self) -> u8 {
        self.g
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }

    pub fn get_color_raw(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    pub fn get_color_raw_mut(&mut self) -> (&mut u8, &mut u8, &mut u8) {
        (&mut self.r, &mut self.g, &mut self.b)
    }

    pub fn get_color(&self) -> Color {
        Color::Rgb{r: self.r, g: self.g, b: self.b}
    }

    pub fn get_x(&self) -> u32 {
        self.position.get_x()
    }

    pub fn get_y(&self) -> u32 {
        self.position.get_y()
    }

    pub fn set_x(&mut self, x: u32) {
        self.position.set_x(x);
    }

    pub fn set_y(&mut self, y: u32) {
        self.position.set_y(y);
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn set_color_raw(&mut self, r: u8, g: u8, b: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
    }

    pub fn set_color(&mut self, color: Color) {
        match color {
            Color::Rgb{r, g, b} => {
                self.r = r;
                self.g = g;
                self.b = b;
            }
            _ => {}
        }
    }

    pub fn quantize(&mut self, shades: u8) {
        let shades = shades as f64;
        let multiplier = 255.0 / shades;
        
        self.r = (((shades * self.r as f64) / 255.0).round() * multiplier) as u8;
        self.g = (((shades * self.g as f64) / 255.0).round() * multiplier) as u8;
        self.b = (((shades * self.b as f64) / 255.0).round() * multiplier) as u8;
    }
}