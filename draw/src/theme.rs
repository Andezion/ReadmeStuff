#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    Matrix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Monospace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    Green,
    Cyan,
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    White,
    Rainbow,
}

impl Tint {
    pub fn hex(self) -> Option<&'static str> {
        Some(match self {
            Tint::Green => "#00ff41",
            Tint::Cyan => "#00e5ff",
            Tint::Blue => "#2f81f7",
            Tint::Purple => "#a371f7",
            Tint::Pink => "#ff4fd8",
            Tint::Red => "#ff3b3b",
            Tint::Orange => "#ff8c00",
            Tint::Yellow => "#ffd600",
            Tint::White => "#e6edf3",
            Tint::Rainbow => return None,
        })
    }

    pub fn color_at(self, x: u32, width: u32) -> String {
        match self.hex() {
            Some(hex) => hex.to_string(),
            None => {
                let hue = if width == 0 {
                    0.0
                } else {
                    (x as f64 / width as f64).clamp(0.0, 1.0) * 360.0
                };
                hsl_to_hex(hue, 1.0, 0.55)
            }
        }
    }

    pub fn recolor(self, hex: &str, x: u32, width: u32) -> String {
        if self == Tint::Green {
            return hex.to_string();
        }
        let Some((_, s, l)) = hex_to_hsl(hex) else {
            return hex.to_string();
        };
        match self {
            Tint::White => hsl_to_hex(0.0, 0.0, l),
            Tint::Rainbow => {
                let hue = if width == 0 {
                    0.0
                } else {
                    (x as f64 / width as f64).clamp(0.0, 1.0) * 360.0
                };
                hsl_to_hex(hue, s, l)
            }
            _ => {
                let target = self.hex().and_then(hex_to_hsl).map_or(0.0, |(h, _, _)| h);
                hsl_to_hex(target, s, l)
            }
        }
    }
}

fn hex_to_hsl(hex: &str) -> Option<(f64, f64, f64)> {
    let raw = hex.strip_prefix('#')?;
    if raw.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(raw, 16).ok()?;
    let r = ((v >> 16) & 0xff) as f64 / 255.0;
    let g = ((v >> 8) & 0xff) as f64 / 255.0;
    let b = (v & 0xff) as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return Some((0.0, 0.0, l));
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    Some((h, s, l))
}

fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h % 360.0) / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let to_u8 = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", to_u8(r), to_u8(g), to_u8(b))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub scheme: ColorScheme,
    pub font: Font,
    pub tint: Tint,
}

impl Theme {
    pub const DARK: Theme = Theme {
        scheme: ColorScheme::Matrix,
        font: Font::Monospace,
        tint: Tint::Green,
    };
}

pub(crate) struct Colors {
    pub bg: &'static str,
    pub border: &'static str,
    pub separator: &'static str,
    pub title: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub accent: &'static str,
    pub matrix_color: Tint,
    pub matrix_opacity: f64,
    pub font_family: &'static str,
}

fn font_family(font: Font) -> &'static str {
    match font {
        Font::Monospace => "monospace",
    }
}

impl Theme {
    pub(crate) fn colors(self) -> Colors {
        let font_family = font_family(self.font);
        match self.scheme {
            ColorScheme::Matrix => Colors {
                bg: "#0d1117",
                border: "#30363d",
                separator: "#21262d",
                title: "#58a6ff",
                text_primary: "#e6edf3",
                text_secondary: "#8b949e",
                accent: self.tint.hex().unwrap_or("#00ff41"),
                matrix_color: self.tint,
                matrix_opacity: 0.22,
                font_family,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_tint_ignores_position() {
        assert_eq!(Tint::Blue.color_at(0, 100), "#2f81f7");
        assert_eq!(Tint::Blue.color_at(99, 100), "#2f81f7");
    }

    #[test]
    fn rainbow_varies_across_the_width() {
        let left = Tint::Rainbow.color_at(0, 300);
        let mid = Tint::Rainbow.color_at(100, 300);
        let right = Tint::Rainbow.color_at(200, 300);
        assert_ne!(left, mid);
        assert_ne!(mid, right);
        assert_eq!(left, "#ff1a1a");
    }

    #[test]
    fn recolor_keeps_green_untouched() {
        assert_eq!(Tint::Green.recolor("#4e7554", 10, 100), "#4e7554");
    }

    #[test]
    fn recolor_shifts_hue_but_keeps_lightness() {
        let (_, _, l_src) = hex_to_hsl("#4e7554").unwrap();
        let blue = Tint::Blue.recolor("#4e7554", 10, 100);
        let (h, _, l) = hex_to_hsl(&blue).unwrap();
        let (target, _, _) = hex_to_hsl("#2f81f7").unwrap();
        assert!((h - target).abs() < 2.0, "hue {h} should be near {target}");
        assert!((l - l_src).abs() < 0.01);
    }

    #[test]
    fn recolor_keeps_intensity_levels_distinct() {
        let levels = ["#4e7554", "#206232", "#185127", "#11371d"];
        for tint in [Tint::Red, Tint::White, Tint::Rainbow] {
            let out: std::collections::HashSet<String> =
                levels.iter().map(|c| tint.recolor(c, 50, 100)).collect();
            assert_eq!(out.len(), levels.len(), "{tint:?} merged levels");
        }
    }

    #[test]
    fn white_recolor_is_grey() {
        let out = Tint::White.recolor("#206232", 0, 100);
        assert_eq!(&out[1..3], &out[3..5]);
        assert_eq!(&out[3..5], &out[5..7]);
    }

    #[test]
    fn accent_follows_solid_tint_and_falls_back_for_rainbow() {
        let blue = Theme {
            tint: Tint::Blue,
            ..Theme::DARK
        };
        assert_eq!(blue.colors().accent, "#2f81f7");
        let rainbow = Theme {
            tint: Tint::Rainbow,
            ..Theme::DARK
        };
        assert_eq!(rainbow.colors().accent, "#00ff41");
    }
}
