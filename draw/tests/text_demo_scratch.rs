use readme_stuff_draw::{Align, Theme, parse_lines, render_text_card};

#[test]
fn manual_render_check() {
    let content = "Hello, World! (32)\nThis is a centered subtitle (18)\nsmaller footer line (12)\n";
    let lines = parse_lines(content);
    assert_eq!(lines.len(), 3);

    for align in [Align::Left, Align::Center, Align::Right] {
        let svg = render_text_card(&lines, align, Theme::Dark);
        let name = match align {
            Align::Left => "left",
            Align::Center => "center",
            Align::Right => "right",
        };
        std::fs::write(
            format!("/tmp/claude-1000/-home-angmar-Desktop-Rust-Labs-ReadmeStuff/6505e87e-2d27-463c-9c56-bb8e11c7197d/scratchpad/text-{name}.svg"),
            &svg,
        )
        .unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<path d="));
    }
}
