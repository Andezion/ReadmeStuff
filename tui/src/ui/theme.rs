use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType};


pub struct Palette {
    pub bg: Color,
    pub border: Color,
    pub title: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
}

pub const PALETTE: Palette = Palette {
    bg: Color::Rgb(0x0d, 0x11, 0x17),
    border: Color::Rgb(0x30, 0x36, 0x3d),
    title: Color::Rgb(0x58, 0xa6, 0xff),
    accent: Color::Rgb(0x00, 0xff, 0x41),
    text_primary: Color::Rgb(0xe6, 0xed, 0xf3),
    text_secondary: Color::Rgb(0x8b, 0x94, 0x9e),
};

pub fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(PALETTE.border))
        .title(Span::styled(
            title,
            Style::default().fg(PALETTE.title).add_modifier(Modifier::BOLD),
        ))
}

pub fn focusable_block(title: &str, focused: bool) -> Block<'_> {
    let color = if focused { PALETTE.accent } else { PALETTE.border };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
        .title(Span::styled(title, Style::default().fg(PALETTE.title)))
}
