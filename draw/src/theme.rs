#[derive(Clone, Copy)]
pub enum ColorScheme {
    Dark,
}

#[derive(Clone, Copy)]
pub enum Font {
    Monospace,
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub scheme: ColorScheme,
    pub font: Font,
}

impl Theme {
    pub const DARK: Theme = Theme {
        scheme: ColorScheme::Dark,
        font: Font::Monospace,
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
    pub matrix_color: &'static str,
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
            ColorScheme::Dark => Colors {
                bg: "#0d1117",
                border: "#30363d",
                separator: "#21262d",
                title: "#58a6ff",
                text_primary: "#e6edf3",
                text_secondary: "#8b949e",
                accent: "#00ff41",
                matrix_color: "#00ff41",
                matrix_opacity: 0.22,
                font_family,
            },
        }
    }
}
