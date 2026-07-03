use ttf_parser::{Face, OutlineBuilder};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Align {
    pub h: HAlign,
    pub v: VAlign,
}

impl Align {
    pub const DEFAULT: Align = Align {
        h: HAlign::Left,
        v: VAlign::Top,
    };

    pub fn parse(word: &str, centered: bool) -> Option<Align> {
        match word.trim().to_lowercase().as_str() {
            "left" => Some(Align {
                h: HAlign::Left,
                v: if centered {
                    VAlign::Center
                } else {
                    VAlign::Top
                },
            }),
            "right" => Some(Align {
                h: HAlign::Right,
                v: if centered {
                    VAlign::Center
                } else {
                    VAlign::Top
                },
            }),
            "top" => Some(Align {
                h: if centered {
                    HAlign::Center
                } else {
                    HAlign::Left
                },
                v: VAlign::Top,
            }),
            "down" | "bottom" => Some(Align {
                h: if centered {
                    HAlign::Center
                } else {
                    HAlign::Left
                },
                v: VAlign::Bottom,
            }),
            "center" | "centre" => Some(Align {
                h: HAlign::Center,
                v: VAlign::Center,
            }),
            _ => None,
        }
    }
}

pub struct TextLine {
    pub text: String,
    pub size: f32,
}

pub struct Font<'a> {
    face: Face<'a>,
}

struct PathBuilder {
    d: String,
    scale: f32,
    dx: f32,
    dy: f32,
}

impl PathBuilder {
    fn tx(&self, x: f32) -> f32 {
        x * self.scale + self.dx
    }

    fn ty(&self, y: f32) -> f32 {
        -y * self.scale + self.dy
    }
}

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.d
            .push_str(&format!("M{:.2} {:.2} ", self.tx(x), self.ty(y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.d
            .push_str(&format!("L{:.2} {:.2} ", self.tx(x), self.ty(y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.d.push_str(&format!(
            "Q{:.2} {:.2} {:.2} {:.2} ",
            self.tx(x1),
            self.ty(y1),
            self.tx(x),
            self.ty(y)
        ));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.d.push_str(&format!(
            "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ",
            self.tx(x1),
            self.ty(y1),
            self.tx(x2),
            self.ty(y2),
            self.tx(x),
            self.ty(y)
        ));
    }

    fn close(&mut self) {
        self.d.push_str("Z ");
    }
}

impl<'a> Font<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Self {
        let face = Face::parse(data, 0).expect("embedded font is invalid TTF/OTF data");
        Font { face }
    }

    fn units_per_em(&self) -> f32 {
        self.face.units_per_em() as f32
    }

    fn advance(&self, ch: char) -> f32 {
        match self.face.glyph_index(ch) {
            Some(id) => self.face.glyph_hor_advance(id).unwrap_or(0) as f32,

            None => self.units_per_em() * 0.4,
        }
    }

    pub fn measure_line(&self, text: &str, font_size: f32) -> f32 {
        let scale = font_size / self.units_per_em();
        text.chars().map(|ch| self.advance(ch) * scale).sum()
    }

    pub fn ascent(&self, font_size: f32) -> f32 {
        self.face.ascender() as f32 * font_size / self.units_per_em()
    }

    pub fn descent(&self, font_size: f32) -> f32 {
        -(self.face.descender() as f32) * font_size / self.units_per_em()
    }

    pub fn line_path(&self, text: &str, font_size: f32, x0: f32, baseline_y: f32) -> String {
        let scale = font_size / self.units_per_em();
        let mut pen_x = x0;
        let mut d = String::new();

        for ch in text.chars() {
            let advance = if let Some(id) = self.face.glyph_index(ch) {
                let mut builder = PathBuilder {
                    d: String::new(),
                    scale,
                    dx: pen_x,
                    dy: baseline_y,
                };
                self.face.outline_glyph(id, &mut builder);
                d.push_str(&builder.d);
                self.face.glyph_hor_advance(id).unwrap_or(0) as f32
            } else {
                self.units_per_em() * 0.4
            };
            pen_x += advance * scale;
        }

        d
    }
}
