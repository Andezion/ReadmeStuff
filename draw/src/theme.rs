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
    pub border: String,
    pub separator: String,
    pub title: String,
    pub text_primary: &'static str,
    pub text_secondary: String,
    pub accent: String,
    pub matrix_color: Tint,
    pub matrix_opacity: f64,
    pub font_family: &'static str,
}

fn font_family(font: Font) -> &'static str {
    match font {
        Font::Monospace => "monospace",
    }
}

#[derive(Clone, Copy, Debug)]
struct Palette {
    hue: f64,
    chroma: f64,
}

impl Palette {
    fn from_tint(tint: Tint) -> Option<Palette> {
        match tint {
            Tint::Rainbow => None,
            Tint::White => Some(Palette {
                hue: 0.0,
                chroma: 0.0,
            }),
            _ => {
                let (hue, _, _) = tint.hex().and_then(hex_to_hsl)?;
                Some(Palette { hue, chroma: 1.0 })
            }
        }
    }

    fn hsl(self, hue_shift: f64, s: f64, l: f64) -> String {
        hsl_to_hex((self.hue + hue_shift).rem_euclid(360.0), s * self.chroma, l)
    }

    fn series(self, i: usize, n: usize) -> String {
        let n = n.max(1);
        let t = if n == 1 {
            0.0
        } else {
            i.min(n - 1) as f64 / (n - 1) as f64
        };
        let drift = if i.is_multiple_of(2) { -1.0 } else { 1.0 } * 10.0 * t;
        self.hsl(drift, 0.95 - 0.35 * t, 0.80 - 0.40 * t)
    }

    fn level(self, t: f64) -> String {
        let t = t.clamp(0.0, 1.0);
        self.hsl(0.0, 0.25 + 0.75 * t, 0.50 + 0.25 * t)
    }
}

impl Colors {
    pub fn series(&self, i: usize, n: usize, original: &str) -> String {
        match Palette::from_tint(self.matrix_color) {
            Some(p) => p.series(i, n),
            None => original.to_string(),
        }
    }

    pub fn ranked(&self, t: f64, original: &str) -> String {
        match Palette::from_tint(self.matrix_color) {
            Some(p) => p.level(t),
            None => original.to_string(),
        }
    }
}

impl Theme {
    pub(crate) fn colors(self) -> Colors {
        let font_family = font_family(self.font);
        match self.scheme {
            ColorScheme::Matrix => {
                let accent = self.tint.hex().unwrap_or("#00ff41").to_string();
                let tinted = |s: f64, l: f64, fallback: &str| match Palette::from_tint(self.tint) {
                    Some(p) => p.hsl(0.0, s, l),
                    None => fallback.to_string(),
                };
                Colors {
                    bg: "#0d1117",
                    border: tinted(0.30, 0.22, "#30363d"),
                    separator: tinted(0.30, 0.15, "#21262d"),
                    title: tinted(0.95, 0.70, "#58a6ff"),
                    text_primary: "#e6edf3",
                    text_secondary: tinted(0.20, 0.62, "#8b949e"),
                    accent,
                    matrix_color: self.tint,
                    matrix_opacity: 0.22,
                    font_family,
                }
            }
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

    fn themed(tint: Tint) -> Colors {
        Theme {
            tint,
            ..Theme::DARK
        }
        .colors()
    }

    #[test]
    fn series_shades_share_the_tint_hue_and_stay_distinct() {
        let c = themed(Tint::Pink);
        let (target, _, _) = hex_to_hsl(Tint::Pink.hex().unwrap()).unwrap();
        let shades: Vec<String> = (0..6).map(|i| c.series(i, 6, "#555555")).collect();
        for s in &shades {
            let (h, _, _) = hex_to_hsl(s).unwrap();
            let d = (h - target).abs().min(360.0 - (h - target).abs());
            assert!(d < 15.0, "{s} hue {h} drifted from {target}");
        }
        let unique: std::collections::HashSet<_> = shades.iter().collect();
        assert_eq!(unique.len(), shades.len());
    }

    #[test]
    fn series_goes_from_bright_to_dim() {
        let c = themed(Tint::Blue);
        let (_, _, first) = hex_to_hsl(&c.series(0, 5, "")).unwrap();
        let (_, _, last) = hex_to_hsl(&c.series(4, 5, "")).unwrap();
        assert!(first > last);
    }

    #[test]
    fn ranked_gets_brighter_with_level() {
        let c = themed(Tint::Red);
        let (_, s_lo, l_lo) = hex_to_hsl(&c.ranked(0.0, "")).unwrap();
        let (_, s_hi, l_hi) = hex_to_hsl(&c.ranked(1.0, "")).unwrap();
        assert!(s_hi > s_lo && l_hi > l_lo);
    }

    #[test]
    fn rainbow_keeps_original_widget_colors() {
        let c = themed(Tint::Rainbow);
        assert_eq!(c.series(1, 3, "#dea584"), "#dea584");
        assert_eq!(c.ranked(0.5, "#42a5f5"), "#42a5f5");
        assert_eq!(c.title, "#58a6ff");
    }

    #[test]
    fn white_tint_uses_greys_everywhere() {
        let c = themed(Tint::White);
        for hex in [
            c.series(2, 4, "#f34b7d"),
            c.ranked(1.0, "#f44336"),
            c.title.clone(),
        ] {
            assert_eq!(&hex[1..3], &hex[3..5], "{hex} is not grey");
            assert_eq!(&hex[3..5], &hex[5..7], "{hex} is not grey");
        }
    }

    #[test]
    fn title_follows_the_tint() {
        let c = themed(Tint::Orange);
        let (h, _, _) = hex_to_hsl(&c.title).unwrap();
        let (target, _, _) = hex_to_hsl(Tint::Orange.hex().unwrap()).unwrap();
        assert!((h - target).abs() < 2.0);
    }
}
